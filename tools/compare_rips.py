"""Pinned native checks for exact Rips construction and supplied flag persistence.

No Python TDA libraries. Each reference computation runs in a fresh process.
This records correctness evidence, not benchmark timing claims.
"""

import argparse
from dataclasses import dataclass, replace
import hashlib
from itertools import combinations
import json
import math
from pathlib import Path
import random
import shlex
import subprocess

from build_native import PINS, REPO, checked_source, instrument_ripser, sha256

WORKERS = REPO / 'tools/reference'


@dataclass
class Fixture:
    name: str
    mode: str
    n: int
    q: int
    cutoff: float | None
    data: list
    precision: str = 'float32-exact'
    characteristic: int = 2

    def write(self, path):
        cutoff = 'none' if self.cutoff is None else repr(self.cutoff)
        lines = [f'{self.mode} {self.n} {self.q} {cutoff} {len(self.data)} {self.characteristic}']
        lines += [repr(v) if self.mode == 'dense' else ' '.join(map(str, v)) for v in self.data]
        path.write_text('\n'.join(lines) + '\n')

    def expected_edges(self):
        if self.mode == 'dense':
            edges = [[a, b, self.data[b * (b - 1) // 2 + a]]
                     for b in range(self.n) for a in range(b)]
        else:
            edges = [[min(a, b), max(a, b), v] for a, b, v in self.data]
        return sorted(e for e in edges if self.cutoff is None or e[2] <= self.cutoff)

    def expected_simplices(self):
        weights = {(a, b): w for a, b, w in self.expected_edges()}
        result = []
        for size in range(1, min(self.q + 2, self.n) + 1):
            for vertices in combinations(range(self.n), size):
                pairs = list(combinations(vertices, 2))
                if all(pair in weights for pair in pairs):
                    result.append([list(vertices), max((weights[pair] for pair in pairs), default=0.)])
        return sorted(result)

    def coverage(self):
        values = self.data if self.mode == 'dense' else [e[2] for e in self.data]
        return self.cutoff if self.cutoff is not None and self.cutoff < max(values, default=0.) else None


def fixtures():
    cases = []
    for q in (0, 1):
        for n in (0, 1):
            for mode in ('dense', 'flag'):
                cases.append(Fixture(f'{mode}_{n}_h{q}', mode, n, q, None, []))
        for cutoff in (None, 0., 1., 1.5, 2.):
            cases.append(Fixture(f'square_{cutoff}_h{q}', 'dense', 4, q, cutoff, [1., 2., 1., 1., 2., 1.]))
            cases.append(Fixture(f'cycle_isolate_{cutoff}_h{q}', 'flag', 5, q, cutoff,
                                 [[0, 1, 1.], [1, 2, 1.], [2, 3, 1.], [0, 3, 1.]]))
        cases.append(Fixture(f'zero_clique_h{q}', 'dense', 5, q, None, [0.] * 10))
        cases.append(Fixture(f'bipartite_h{q}', 'flag', 12, q, None,
                             [[a, b, 1.] for a in range(5) for b in range(5, 11)]))
        cases.append(Fixture(f'adjacent_f64_h{q}', 'dense', 4, q, 1.,
                             [1., math.nextafter(1., 2.), 1., 1., math.nextafter(1., 2.), 1.], 'float64'))
    rng = random.Random(20260920)
    for sample in range(32):
        n = rng.randrange(2, 15)
        q = sample % 2
        cutoff = (None, 0., .5, 1.)[sample % 4]
        values = [rng.randrange(9) / 4 for _ in range(n * (n - 1) // 2)]
        edges = [[a, b, values[b * (b - 1) // 2 + a]] for b in range(n) for a in range(b)
                 if rng.randrange(3) != 0]
        cases.append(Fixture(f'random_dense_{sample}', 'dense', n, q, cutoff, values))
        cases.append(Fixture(f'random_flag_{sample}', 'flag', n, q, cutoff, edges))
    for q in (2, 3, 4):
        for n in (0, 1):
            for mode in ('dense', 'flag'):
                cases.append(Fixture(f'{mode}_{n}_h{q}', mode, n, q, None, []))
        n = 2 * (q + 1)
        values = [2. if a // 2 == b // 2 else 1. for b in range(n) for a in range(b)]
        for cutoff in (None, 1., 2.):
            cases.append(Fixture(f'sphere_{cutoff}_h{q}', 'dense', n, q, cutoff, values))
        edges = [[a, b, 1.] for b in range(n) for a in range(b) if a // 2 != b // 2]
        cases.append(Fixture(f'supplied_sphere_h{q}', 'flag', n + 1, q, None, edges))
        cases.append(Fixture(f'zero_clique_h{q}', 'dense', 7, q, None, [0.] * 21))
    for sample in range(48):
        n = rng.randrange(4, 10)
        q = 2 + sample % 3
        cutoff = (None, 0., .5, 1.)[sample % 4]
        values = [rng.randrange(9) / 4 for _ in range(n * (n - 1) // 2)]
        edges = [[a, b, values[b * (b - 1) // 2 + a]] for b in range(n) for a in range(b)
                 if rng.randrange(4) != 0]
        cases.append(Fixture(f'higher_dense_{sample}', 'dense', n, q, cutoff, values))
        cases.append(Fixture(f'higher_flag_{sample}', 'flag', n, q, cutoff, edges))
    base = [case for case in cases if case.n <= 9 and case.precision == 'float32-exact']
    for p in (3, 5):
        cases += [replace(case, name=f'{case.name}_p{p}', characteristic=p) for case in base]
    facets = [(0, 1, 2), (0, 1, 3), (0, 2, 4), (0, 3, 5), (0, 4, 5),
              (1, 2, 5), (1, 3, 4), (1, 4, 5), (2, 3, 4), (2, 3, 5)]
    faces = sorted({face for facet in facets for size in (1, 2, 3) for face in combinations(facet, size)})
    edges = [[a, b, 1.] for b in range(len(faces)) for a in range(b)
             if set(faces[a]) < set(faces[b]) or set(faces[b]) < set(faces[a])]
    for p in (2, 3, 5, 251, 46337, 4294967291):
        cases.append(Fixture(f'projective_plane_p{p}', 'flag', len(faces), 2, None, edges, characteristic=p))
    for p in (251, 46337, 4294967291):
        cases.append(Fixture(f'square_p{p}', 'dense', 4, 1, None, [1., 2., 1., 1., 2., 1.], characteristic=p))
    return cases


def canonical_intervals(rows):
    # Preserve multiplicity; never compare sets or rounded printed endpoints.
    return sorted(rows, key=lambda row: (row[0], row[1], math.inf if row[2] is None else row[2]))


def compare(case, actual, reference):
    if actual['status'] != 'ok' or reference['status'] != 'ok':
        raise ValueError('unsupported/error result cannot count as a comparison')
    if actual.get('characteristic') != case.characteristic or reference.get('characteristic') != case.characteristic:
        raise ValueError(f'{case.name}: coefficient field mismatch')
    expected = case.expected_edges()
    if sorted(actual['edges']) != expected or sorted(reference['edges']) != expected:
        raise ValueError(f'{case.name}: graph mismatch')
    if 'simplices' in reference:
        expected_simplices = case.expected_simplices()
        if sorted(actual.get('simplices', [])) != expected_simplices or sorted(reference['simplices']) != expected_simplices:
            raise ValueError(f'{case.name}: expanded simplex/value mismatch')
    if actual['coverage'] != case.coverage():
        raise ValueError(f'{case.name}: original-source coverage mismatch')
    if canonical_intervals(actual['intervals']) != canonical_intervals(reference['intervals']):
        raise ValueError(f'{case.name}: interval multiset mismatch')


def reference_exclusion(name, case):
    if name == 'gudhi' and case.characteristic > 46337:
        return 'pinned Field_Zp supports primes at most 46337'
    if name == 'ripser':
        if case.characteristic > 251:
            return 'pinned coefficient bit width supports primes at most 251'
        if case.precision == 'float64':
            return 'fixture requires float64; upstream value_t is float'
        if case.n < 2:
            return 'upstream engine requires at least two vertices in this adapter'
    return None


def check_field_protocol(build, fixture_dir):
    """Exercise real adapters: malformed fields must never fall back to F2."""
    checks = []
    tokens = ['0', '1', '4', '4294967295', '4294967296', '-3', '3.5', 'invalid', '3 5']
    for i, token in enumerate(tokens):
        path = fixture_dir / f'protocol_invalid_{i}.txt'
        path.write_text(f'flag 2 0 none 0 {token}\n')
        for name in ('cocycle', 'gudhi', 'ripser'):
            process = subprocess.run([str(build / name), str(path)], capture_output=True, text=True, timeout=10)
            if process.returncode == 0:
                raise ValueError(f'{name} silently accepted invalid field {token}')
            checks.append({'worker': name, 'field_token': token, 'status': 'rejected', 'stderr': process.stderr})
    for characteristic in (2, 257, 65537, 4294967291):
        path = fixture_dir / f'protocol_valid_{characteristic}.txt'
        # The F2 case specifically checks backward-compatible five-field headers.
        suffix = '' if characteristic == 2 else f' {characteristic}'
        path.write_text(f'flag 2 0 none 0{suffix}\n')
        case = Fixture('field_protocol', 'flag', 2, 0, None, [], characteristic=characteristic)
        for name in ('cocycle', 'gudhi', 'ripser'):
            process = subprocess.run([str(build / name), str(path)], capture_output=True, text=True, timeout=10, check=True)
            output = json.loads(process.stdout)
            exclusion = reference_exclusion(name, case)
            if exclusion:
                if output['status'] != 'unsupported':
                    raise ValueError(f'{name} failed to enforce its coefficient limit')
            elif output['status'] != 'ok' or output['characteristic'] != characteristic:
                raise ValueError(f'{name} changed the declared field')
            checks.append({'worker': name, 'characteristic': characteristic, 'status': output['status']})
    return checks


def source_hash():
    paths = [REPO / 'Cargo.toml', REPO / 'Cargo.lock', *sorted((REPO / 'src').rglob('*.rs')),
             *sorted(WORKERS.glob('*')), Path(__file__).resolve(), REPO / 'tools/build_native.py',
             REPO / 'benches/native/sources.json']
    digest = hashlib.sha256()
    for path in paths:
        digest.update(path.relative_to(REPO).as_posix().encode() + b'\0' + path.read_bytes())
    return digest.hexdigest()


def run(args):
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    build = output / 'build'
    build.mkdir()
    fixture_dir = output / 'fixtures'
    fixture_dir.mkdir()
    initial_hash = source_hash()
    gudhi = checked_source(args.gudhi_source, 'gudhi')
    ripser = checked_source(args.ripser_source, 'ripser')
    if sha256(ripser / 'ripser.cpp') != PINS['ripser']['source_sha256']:
        raise ValueError('Ripser source bytes differ from pin')
    (build / 'ripser_instrumented.cpp').write_text(instrument_ripser((ripser / 'ripser.cpp').read_text()))
    commands = []

    def compile_worker(command):
        commands.append(command)
        with (build / 'build.log').open('a') as log:
            log.write(shlex.join(command) + '\n')
            log.flush()
            subprocess.run(command, cwd=REPO, stdout=log, stderr=subprocess.STDOUT, check=True, timeout=180)

    cargo = build / 'cargo'
    compile_worker(['cargo', 'build', '--locked', '--offline', '--release', '--lib', '--target-dir', str(cargo)])
    compile_worker(['rustc', '--edition=2024', '-D', 'warnings', '-O', str(WORKERS / 'rips_cocycle.rs'),
                    '--extern', f'cocycle={cargo}/release/libcocycle.rlib', '-L', f'dependency={cargo}/release/deps',
                    '-o', str(build / 'cocycle')])
    flags = ['-std=c++17', '-O2', '-DNDEBUG', '-MMD', '-I', str(build)]
    if args.boost_include:
        flags += ['-I', str(args.boost_include.resolve())]
    for include in sorted(gudhi.glob('src/*/include')):
        flags += ['-I', str(include)]
    for name in ('gudhi', 'ripser'):
        compile_worker([args.cxx, *flags, '-MF', str(build / f'{name}.d'),
                        str(WORKERS / f'rips_{name}.cpp'), '-o', str(build / name)])
    dependencies = {}
    for name in ('gudhi', 'ripser'):
        dependency_text = (build / f'{name}.d').read_text().replace('\\\n', '')
        for filename in shlex.split(dependency_text.split(':', 1)[1]):
            dependencies[filename] = sha256(Path(filename))
    protocol_checks = check_field_protocol(build, fixture_dir)
    metadata = {'protocol_checks': protocol_checks, 'pins': PINS, 'source_sha256': initial_hash, 'commands': commands,
                'dependency_sha256': dependencies,
                'compiler': subprocess.check_output([args.cxx, '--version'], text=True),
                'rustc': subprocess.check_output(['rustc', '-Vv'], text=True),
                'binary_sha256': {name: sha256(build / name) for name in ('cocycle', 'gudhi', 'ripser')}}
    (output / 'environment.json').write_text(json.dumps(metadata, indent=2) + '\n')
    results = []
    comparisons = 0
    exclusions = 0
    for case in fixtures():
        path = fixture_dir / f'{case.name}.txt'
        case.write(path)
        record = {'name': case.name, 'precision': case.precision, 'characteristic': case.characteristic, 'fixture_sha256': sha256(path), 'outputs': {}}
        results.append(record)
        try:
            for name in ('cocycle', 'gudhi', 'ripser'):
                if reason := reference_exclusion(name, case):
                    record['outputs'][name] = {'status': 'unsupported', 'reason': reason}
                else:
                    process = subprocess.run([str(build / name), str(path)], capture_output=True, text=True, timeout=30)
                    if process.returncode:
                        raise ValueError(f'{case.name}/{name}: {process.stderr}')
                    record['outputs'][name] = json.loads(process.stdout)
            # Also validate Rust-only large-field cases without claiming native parity.
            actual = record['outputs']['cocycle']
            compare(case, actual, actual)
            for name in ('gudhi', 'ripser'):
                reference = record['outputs'][name]
                if reference['status'] == 'unsupported':
                    if not reference_exclusion(name, case):
                        raise ValueError(f'unexpected exclusion: {case.name}/{name}')
                    exclusions += 1
                    continue
                compare(case, record['outputs']['cocycle'], reference)
                comparisons += 1
            record['status'] = 'passed'
        except Exception as error:
            record['status'] = 'failed'
            record['error'] = str(error)
            raise
        finally:
            (output / 'results.json').write_text(json.dumps(results, indent=2, allow_nan=False) + '\n')
    if source_hash() != initial_hash:
        raise ValueError('source changed during comparison; use a fresh output directory')
    summary = {'status': 'passed', 'protocol_checks': len(protocol_checks), 'cases': len(results), 'comparisons': comparisons, 'exclusions': exclusions,
               'source_sha256': initial_hash}
    (output / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n')
    print(json.dumps(summary))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--gudhi-source', type=Path, default=REPO / 'target/native-sources/gudhi')
    parser.add_argument('--ripser-source', type=Path, default=REPO / 'target/native-sources/ripser')
    parser.add_argument('--boost-include', type=Path)
    parser.add_argument('--cxx', default='c++')
    run(parser.parse_args())


if __name__ == '__main__':
    main()
