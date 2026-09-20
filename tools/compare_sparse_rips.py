"""Native sparse Rips agreement with pinned GUDHI C++ (no Python TDA).

Only the sampling call is replaced, to fix start/tie semantics independently of
Rust. Upstream edges, high-dimensional blockers and cohomology remain unchanged.
Unmodified upstream metric sampling is additionally checked on unique choices.
Ripser has no equivalent sparse approximation constructor and is not an oracle
for blocker-aware topology; ordinary exact/flag checks live in compare_rips.py.
"""
import argparse
import hashlib
from itertools import combinations
import json
from pathlib import Path
import random
import shlex
import subprocess

from build_native import PINS, REPO, checked_source, sha256
from compare_rips import Fixture, source_hash


def instrument_sparse(source):
    token = 'subsampling::choose_n_farthest_points_metric('
    if source.count(token) != 1:
        raise ValueError('expected exactly one upstream sparse sampling call')
    return source.replace(token, '::sparse_reference::sample(')


def fingerprint():
    return hashlib.sha256(source_hash().encode() + Path(__file__).read_bytes()).hexdigest()


def cases():
    samples = [[], [(0, 0)], [(0, 0)] * 4,
               [(0, 0), (0, 0), (2, 0), (10, 0), (5, 0)],
               [(68, 71), (87, 26), (42, 69), (15, 91), (81, 8), (39, 52), (10, 64), (60, 81)]]
    rng = random.Random(20260920)
    samples += [[(rng.randrange(300), rng.randrange(300)) for _ in range(rng.randrange(3, 9))]
                for _ in range(20)]
    for sample, points in enumerate(samples):
        n = len(points)
        values = [float(sum(abs(x - y) for x, y in zip(points[a], points[b])))
                  for b in range(n) for a in range(b)]
        for variant, epsilon in enumerate((0.125, 0.5, 0.875, 1., 1.5, 2.)):
            q = (sample + variant) % 5
            cutoff = (None, 0., 25., 100.)[(sample + variant) % 4]
            minimum = (0., 3., 20.)[variant % 3]
            start = (sample + variant) % n if n else 0
            p = (2, 3, 5, 251, 46337)[q]
            fixture = Fixture(f'sparse_{sample}_{variant}', 'dense', n, q, cutoff, values,
                              'float64', p)
            yield fixture, epsilon, minimum, start, q + 1
    # Explicit dimension zero and the blocker counterexample must not disappear
    # behind the random suite's scale truncation or changed start.
    points = samples[4]
    values = [float(sum(abs(x - y) for x, y in zip(points[a], points[b])))
              for b in range(8) for a in range(b)]
    for dimension in (0, 1, 2, 3, 7):
        yield Fixture(f'blocker_dim_{dimension}', 'dense', 8, 2, None, values, 'float64', 3), .5, 0., 0, dimension
    # Adjacent binary64 scales remain exact comparisons, without output rounding.
    yield Fixture('adjacent_f64', 'dense', 3, 1, None, [1., 2., 1.0000000000000002], 'float64', 5), .5, 0., 0, 2


def independent_simplices(case, epsilon, minimum, order, radii, dimension):
    """Subset-based topology check, independent of recursive clique traversal."""
    retained = []
    for v, radius in zip(order, radii):
        if radius is not None and (radius <= 0 or radius < minimum):
            break
        retained.append(v)
    radius = dict(zip(order, radii))
    edges = {}
    excluded_scale = False
    for i, a in enumerate(retained):
        for b in retained[i + 1:]:
            lo, hi = sorted((a, b))
            d = case.data[hi * (hi - 1) // 2 + lo]
            li = float('inf') if radius[a] is None else radius[a]
            lj = radius[b]
            if d * epsilon <= 2 * lj:
                value = d
            elif d * epsilon > li + lj:
                continue
            else:
                value = 2 * (d - lj / epsilon)
                if epsilon < 1 and value * (epsilon * (1 - epsilon) / 2) > lj:
                    continue
            if case.cutoff is not None and value > case.cutoff:
                excluded_scale = True
            else:
                edges[lo, hi] = value
    simplices = []
    for size in range(1, min(dimension + 1, len(retained)) + 1):
        for vertices in combinations(sorted(retained), size):
            pairs = list(combinations(vertices, 2))
            if not all(pair in edges for pair in pairs):
                continue
            value = max((edges[pair] for pair in pairs), default=0.)
            if size > 2 and epsilon < 1 and any(
                    radius[v] is not None and radius[v] < value * (epsilon * (1 - epsilon) / 2) for v in vertices):
                continue
            simplices.append([list(vertices), value])
    return sorted(simplices), case.cutoff if excluded_scale else None


def run(args):
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    build = output / 'build'
    fixtures = output / 'fixtures'
    build.mkdir()
    fixtures.mkdir()
    initial_hash = fingerprint()
    gudhi = checked_source(args.gudhi_source, 'gudhi')
    upstream = gudhi / 'src/Rips_complex/include/gudhi/Sparse_rips_complex.h'
    instrumented = build / 'Sparse_rips_instrumented.h'
    instrumented.write_text(instrument_sparse(upstream.read_text()))
    commands = []

    def compile_worker(command):
        commands.append(command)
        with (build / 'build.log').open('a') as log:
            log.write(shlex.join(command) + '\n')
            log.flush()
            subprocess.run(command, cwd=REPO, stdout=log, stderr=subprocess.STDOUT, check=True, timeout=180)

    cargo = build / 'cargo'
    compile_worker(['cargo', 'build', '--locked', '--offline', '--release', '--lib', '--target-dir', str(cargo)])
    compile_worker(['rustc', '--edition=2024', '-D', 'warnings', '-O', str(REPO / 'tools/reference/sparse_cocycle.rs'),
                    '--extern', f'cocycle={cargo}/release/libcocycle.rlib', '-L', f'dependency={cargo}/release/deps',
                    '-o', str(build / 'cocycle')])
    flags = ['-std=c++17', '-O2', '-DNDEBUG', '-MMD', '-MF', str(build / 'gudhi.d'), '-I', str(build)]
    if args.boost_include:
        flags += ['-I', str(args.boost_include.resolve())]
    for include in sorted(gudhi.glob('src/*/include')):
        flags += ['-I', str(include)]
    compile_worker([args.cxx, *flags, str(REPO / 'tools/reference/sparse_gudhi.cpp'), '-o', str(build / 'gudhi')])
    dependency_text = (build / 'gudhi.d').read_text().replace('\\\n', '')
    dependencies = {filename: sha256(Path(filename)) for filename in shlex.split(dependency_text.split(':', 1)[1])}
    metadata = {'pin': PINS['gudhi'], 'source_sha256': initial_hash,
                'upstream_header_sha256': sha256(upstream), 'instrumented_header_sha256': sha256(instrumented),
                'instrumentation': 'only constructor sampling call replaced; dim-0 tree pruned to vertices in adapter',
                'sampling': 'independent exhaustive deterministic start/ties; original metric routine checked when positive choices unique',
                'ripser': 'excluded: no sparse Rips approximation with higher-simplex blockers',
                'commands': commands, 'dependency_sha256': dependencies,
                'compiler': subprocess.check_output([args.cxx, '--version'], text=True),
                'rustc': subprocess.check_output(['rustc', '-Vv'], text=True),
                'binary_sha256': {name: sha256(build / name) for name in ('cocycle', 'gudhi')}}
    (output / 'environment.json').write_text(json.dumps(metadata, indent=2) + '\n')
    results = []
    checked_metric = 0
    persistence_comparisons = 0
    try:
        for case, epsilon, minimum, start, dimension in cases():
            path = fixtures / f'{case.name}.txt'
            case.write(path)
            record = {'name': case.name, 'epsilon': epsilon, 'min_insertion_radius': minimum,
                      'start': start, 'dimension': dimension, 'fixture_sha256': sha256(path), 'outputs': {}}
            results.append(record)
            outputs = record['outputs']
            for name in ('cocycle', 'gudhi'):
                command = [str(build / name), str(path), str(epsilon), str(minimum), str(start), str(dimension)]
                completed = subprocess.run(command, check=True, text=True, capture_output=True, timeout=60)
                topology, sampling = map(json.loads, completed.stdout.splitlines())
                outputs[name] = {'topology': topology, 'sampling': sampling}
            rust, native = outputs['cocycle'], outputs['gudhi']
            assert rust['sampling']['permutation'] == native['sampling']['permutation'], case.name
            assert rust['sampling']['radii'] == native['sampling']['radii'], case.name
            expected, coverage = independent_simplices(case, epsilon, minimum, **{
                'order': rust['sampling']['permutation'], 'radii': rust['sampling']['radii'], 'dimension': dimension})
            assert expected == sorted(rust['topology']['simplices']) == sorted(native['topology']['simplices']), case.name
            assert rust['topology']['coverage'] == coverage, case.name
            interval_key = lambda i: (i[0], i[1], float('inf') if i[2] is None else i[2])
            assert sorted(rust['topology']['intervals'], key=interval_key) == sorted(native['topology']['intervals'], key=interval_key), case.name
            assert rust['topology']['characteristic'] == native['topology']['characteristic'] == case.characteristic
            checked_metric += native['sampling']['metric_sampling_checked']
            persistence_comparisons += dimension > case.q or dimension >= case.n
            record['status'] = 'passed'
        if fingerprint() != initial_hash:
            raise ValueError('sources changed during comparison')
        summary = {'status': 'passed', 'cases': len(results), 'topology_comparisons': len(results),
                   'persistence_comparisons': persistence_comparisons,
                   'unmodified_metric_sampling_checks': checked_metric,
                   'source_sha256': initial_hash, 'ripser': metadata['ripser']}
        (output / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n')
        print(json.dumps(summary))
    finally:
        (output / 'results.json').write_text(json.dumps(results, indent=2) + '\n')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', required=True, type=Path)
    parser.add_argument('--gudhi-source', type=Path, default=REPO / 'target/native-sources/gudhi')
    parser.add_argument('--boost-include', type=Path)
    parser.add_argument('--cxx', default='g++')
    run(parser.parse_args())


if __name__ == '__main__':
    main()
