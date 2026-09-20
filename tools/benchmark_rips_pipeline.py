"""Measure public Rips workflows and native C++ references in fresh Linux processes.

Five bins: input validation/conversion, construction, expansion, compute, export.
Public Rust compute includes owned diagram normalization; GUDHI extraction is in
export and Ripser captures pairs during compute. End-to-end time includes every
bin plus intervening work, excludes fixture parsing and final JSON transport.
"""
import argparse
from dataclasses import dataclass, replace
from datetime import datetime, timezone
import hashlib
import json
import math
import os
from pathlib import Path
import platform
import random
import shlex
import statistics
import subprocess
import sys

from benchmark_inputs import distances, uniform
from build_native import PINS, REPO, checked_source, instrument_ripser, sha256
from compare_rips import Fixture

PIPELINE = REPO / 'benches/pipeline'
PROTOCOL = 'cocycle-rips-pipeline-v2'
PHASES = ['input', 'construction', 'expansion', 'compute', 'export']


@dataclass
class Case:
    fixture: Fixture
    path: str
    layout: str = 'lower'
    epsilon: float = .5
    representatives: bool = False
    coordinates: list | None = None

    @property
    def name(self):
        return f'{self.fixture.name}_{self.path}_{self.layout}_{"bases" if self.representatives else "diagram"}'


def greedy_unique(values, n):
    order = []
    for _ in range(n):
        if not order:
            order.append(0)
            continue
        nearest = []
        for v in range(n):
            if v in order:
                continue
            distance = min(values[max(v, u) * (max(v, u) - 1) // 2 + min(v, u)] for u in order)
            nearest.append((distance, -v, v))
        nearest.sort(reverse=True)
        if len(nearest) > 1 and nearest[0][0] == nearest[1][0]:
            return None
        order.append(nearest[0][2])
    return order


def cases(quick):
    import struct

    def f32(v):
        return struct.unpack('<f', struct.pack('<f', v))[0]

    n = 24 if quick else 64
    circle = [(math.cos(2 * math.pi * i / n), math.sin(2 * math.pi * i / n)) for i in range(n)]
    base = Fixture(f'circle{n}', 'dense', n, 1, None, list(map(f32, distances(circle))))
    yield Case(base, 'dense')
    yield Case(base, 'dense', 'upper')
    yield Case(base, 'dense', 'square')
    limited = replace(base, name=base.name + '_cutoff', cutoff=f32(.4))
    yield Case(limited, 'threshold')
    yield Case(limited, 'expanded')
    yield Case(replace(limited, characteristic=3, name=limited.name + '_f3'), 'threshold')
    small = 12 if quick else 24
    rng = random.Random(20260920)
    nonmetric = Fixture(f'nonmetric{small}', 'dense', small, 2, None,
                        [rng.randrange(1, 17) / 16 for _ in range(small * (small - 1) // 2)], characteristic=3)
    yield Case(nonmetric, 'dense')
    yield Case(nonmetric, 'expanded')
    sphere = Fixture('sphere_h2', 'dense', 6, 2, 1.,
                     [2. if a // 2 == b // 2 else 1. for b in range(6) for a in range(b)], characteristic=3)
    yield Case(sphere, 'threshold')
    yield Case(sphere, 'expanded')
    yield Case(sphere, 'threshold', representatives=True)
    # H1 at cutoff has an active basis; reference timing does not include bases.
    yield Case(limited, 'threshold', representatives=True)
    size = 6 if quick else 16
    bipartite = Fixture(f'bipartite{size}', 'flag', 2 * size, 1, None,
                        [[a, b, 1.] for a in range(size) for b in range(size, 2 * size)])
    yield Case(bipartite, 'flag')
    yield Case(replace(bipartite, characteristic=3, name=bipartite.name + '_f3'), 'flag')
    yield Case(replace(bipartite, n=1000 if quick else 10000, name='bipartite_isolates'), 'flag')
    for size in ([24] if quick else [32, 64, 128]):
        points = uniform(size)
        yield Case(Fixture(f'uniform{size}_h0', 'dense', size, 0, None,
                          list(map(f32, distances(points)))), 'dense')
        # Dyadic Manhattan distances are exact metric values, also float32-exact.
        for _ in range(100):
            points = [(rng.randrange(1 << 20), rng.randrange(1 << 20)) for _ in range(size)]
            values = [sum(abs(x - y) for x, y in zip(points[a], points[b])) / (1 << 20)
                      for b in range(size) for a in range(b)]
            if greedy_unique(values, size) is not None:
                break
        else:
            raise ValueError('could not generate unique greedy choices')
        approximate = Fixture(f'metric{size}', 'dense', size, 1, None, values, characteristic=3)
        yield Case(approximate, 'approximate')
        if size == (24 if quick else 64):
            yield Case(approximate, 'approximate_checked')
            yield Case(approximate, 'approximate_expanded')
            yield Case(approximate, 'approximate', representatives=True)
    # Native distance workers receive exactly the same line metric. Their input
    # is precomputed, so their timings are correctness references, not point timings.
    points = [(float(i * i), 0.) for i in range(n)]
    yield Case(Fixture('points_line', 'dense', n, 1, float(n), distances(points)),
               'points', coordinates=[x for point in points for x in point])


def source_hash():
    paths = [REPO / 'Cargo.toml', REPO / 'Cargo.lock', *sorted((REPO / 'src').rglob('*.rs')),
             *sorted(PIPELINE.glob('*')), REPO / 'tools/reference/rips_common.hpp',
             Path(__file__).resolve(), REPO / 'tools/build_native.py', REPO / 'tools/compare_rips.py',
             REPO / 'tools/benchmark_inputs.py', REPO / 'benches/native/sources.json']
    digest = hashlib.sha256()
    for p in paths:
        digest.update(p.relative_to(REPO).as_posix().encode() + b'\0' + p.read_bytes())
    return digest.hexdigest()


def instrument_sparse(source):
    old = 'boost::size(points)), -1, -1,'
    marker = '  Graph graph_;'
    if source.count(old) != 1 or source.count(marker) != 1:
        raise ValueError('sparse header instrumentation no longer matches pin')
    # Keep the real metric sampler; only choose a reproducible start and expose
    # its result so the controller can refuse unequal approximate constructions.
    return source.replace(old, 'boost::size(points)), -1, 0,').replace(marker,
        ' public:\n  const auto& benchmark_permutation() const { return sorted_points; }\n private:\n' + marker)


def build(args, output):
    directory = output / 'build'
    directory.mkdir()
    gudhi = checked_source(args.gudhi_source, 'gudhi')
    ripser = checked_source(args.ripser_source, 'ripser')
    if sha256(ripser / 'ripser.cpp') != PINS['ripser']['source_sha256']:
        raise ValueError('Ripser source bytes differ from pin')
    (directory / 'ripser_instrumented.cpp').write_text(instrument_ripser((ripser / 'ripser.cpp').read_text()))
    sparse = gudhi / 'src/Rips_complex/include/gudhi/Sparse_rips_complex.h'
    (directory / 'Sparse_rips_fixed_start.h').write_text(instrument_sparse(sparse.read_text()))
    commands = []

    def compile_worker(command):
        commands.append(command)
        with (directory / 'build.log').open('a') as log:
            log.write(shlex.join(command) + '\n')
            log.flush()
            subprocess.run(command, cwd=REPO, check=True, stdout=log, stderr=subprocess.STDOUT, timeout=180)

    target = directory / 'cargo'
    compile_worker(['cargo', 'build', '--release', '--locked', '--offline', '--lib', '--target-dir', str(target)])
    compile_worker(['rustc', '--edition=2024', '-O', '-D', 'warnings', str(PIPELINE / 'cocycle.rs'),
                    '--extern', f'cocycle={target}/release/libcocycle.rlib', '-L', f'dependency={target}/release/deps',
                    '-o', str(directory / 'cocycle')])
    flags = ['-std=c++17', '-O3', '-DNDEBUG', '-MMD', '-I', str(directory)]
    if args.boost_include:
        flags += ['-I', str(args.boost_include.resolve())]
    for include in sorted(gudhi.glob('src/*/include')):
        flags += ['-I', str(include)]
    for name in ('gudhi', 'ripser'):
        compile_worker([args.cxx, *flags, '-MF', str(directory / f'{name}.d'),
                        str(PIPELINE / f'{name}.cpp'), '-o', str(directory / name)])
    dependencies = {}
    for name in ('gudhi', 'ripser'):
        text = (directory / f'{name}.d').read_text().replace('\\\n', '')
        for filename in shlex.split(text.split(':', 1)[1]):
            dependencies[filename] = sha256(Path(filename))
    return {'commands': commands, 'dependencies_sha256': dependencies, 'pins': PINS,
            'binaries_sha256': {n: sha256(directory / n) for n in ('cocycle', 'gudhi', 'ripser')},
            'sparse_header_original_sha256': sha256(sparse),
            'instrumentation': 'GUDHI fixed start 0 plus read-only permutation accessor; Ripser numeric interval output hook',
            'rustc': subprocess.check_output(['rustc', '-Vv'], text=True),
            'cxx': subprocess.check_output([args.cxx, '--version'], text=True)}


def exclusion(case, backend):
    if backend == 'ripser' and case.path.startswith('approximate'):
        return 'no sparse approximation constructor or higher-simplex blockers'
    if backend == 'ripser' and case.fixture.n < 2:
        return 'pinned adapter requires at least two vertices'
    if backend == 'ripser' and case.fixture.characteristic > 251:
        return 'coefficient limit 251'
    if backend == 'gudhi' and case.fixture.characteristic > 46337:
        return 'coefficient limit 46337'
    return None


def timing_scope(case, backend):
    if backend == 'cocycle':
        return 'workflow'
    if (case.representatives or case.path in ('points', 'approximate_checked')
            or case.layout != 'lower' or (backend == 'ripser' and case.path == 'expanded')):
        return 'correctness_reference_only'
    return 'workflow'


def validate_output(data, case):
    if not isinstance(data, dict):
        raise ValueError('worker output must be an object')
    if data.get('status') != 'completed':
        raise ValueError('worker did not complete')
    for v in [data['elapsed_ms'], *data['phases_ms']]:
        if type(v) not in (int, float) or not math.isfinite(v) or v < 0:
            raise ValueError('invalid duration')
    if data['elapsed_ms'] <= 0:
        raise ValueError('end-to-end duration must be positive')
    if len(data['phases_ms']) != 5 or sum(data['phases_ms']) > data['elapsed_ms'] + .001:
        raise ValueError('phase times exceed end-to-end time')
    for key in ('rss_before_kib', 'hwm_before_kib', 'peak_rss_kib'):
        if type(data[key]) is not int or data[key] <= 0:
            raise ValueError('missing process memory')
    if data['peak_rss_kib'] < data['hwm_before_kib']:
        raise ValueError('peak decreased')
    for row in data['intervals']:
        if len(row) != 3 or type(row[0]) is not int or not 0 <= row[0] <= case.fixture.q:
            raise ValueError('bad interval dimension')
        _, birth, death = row
        if type(birth) not in (int, float) or not math.isfinite(birth) or birth < 0:
            raise ValueError('bad birth')
        if death is not None and (type(death) not in (int, float) or not math.isfinite(death) or death <= birth):
            raise ValueError('bad death')
        if case.fixture.cutoff is not None and (birth > case.fixture.cutoff or (death is not None and death > case.fixture.cutoff)):
            raise ValueError('interval outside cutoff')


def worker(command, case, timeout, memory_mib, cpu=None):
    import resource

    def limits():
        limit = memory_mib * 1024 * 1024
        resource.setrlimit(resource.RLIMIT_AS, (limit, limit))
        if cpu is not None:
            os.sched_setaffinity(0, {cpu})

    try:
        result = subprocess.run(command, capture_output=True, text=True, timeout=timeout, preexec_fn=limits)
    except subprocess.TimeoutExpired as error:
        return {'status': 'timeout', 'timeout_seconds': timeout, 'stderr': str(error.stderr)[-2000:]}
    except OSError as error:
        return {'status': 'process_error', 'error': repr(error)}
    if result.returncode:
        return {'status': 'process_error', 'returncode': result.returncode, 'stderr': result.stderr[-8000:]}
    try:
        data = json.loads(result.stdout)
        validate_output(data, case)
        return data
    except (KeyError, ValueError, TypeError) as error:
        return {'status': 'protocol_error', 'error': str(error), 'stdout': result.stdout[-2000:]}


def canonical(data):
    return sorted(data['intervals'], key=lambda i: (i[0], i[1], math.inf if i[2] is None else i[2]))


def compare_samples(anchor, other, case):
    if canonical(anchor) != canonical(other):
        raise ValueError('interval multiset mismatch')
    if case.path.startswith('approximate'):
        expected = greedy_unique(case.fixture.data, case.fixture.n)
        if expected is None or anchor['permutation'] != expected or other['permutation'] != expected:
            raise ValueError('approximation sampling differs')
    if 'coverage' in anchor and 'coverage' in other and anchor['coverage'] != other['coverage']:
        raise ValueError('coverage changed between samples')
    if anchor.get('representative_terms') is not None and other.get('representative_terms') is not None and anchor['representative_terms'] != other['representative_terms']:
        raise ValueError('representative payload changed between samples')
    if anchor.get('simplices') is not None and other.get('simplices') and anchor['simplices'] != other['simplices']:
        raise ValueError('expanded simplex count mismatch')


def provenance(kernel_revision):
    def git(*args):
        return subprocess.check_output(['git', *args], cwd=REPO, text=True).strip()

    dirty = git('status', '--porcelain', '--untracked-files=normal')
    if dirty:
        raise ValueError('commit or stash changes before a version-bound measurement')
    harness = git('rev-parse', 'HEAD')
    kernel = git('rev-parse', '--verify', kernel_revision + '^{commit}')
    if git('diff', kernel, '--', 'Cargo.toml', 'Cargo.lock', 'src'):
        raise ValueError('current Rust kernel differs from --kernel-revision')
    return {'harness_commit': harness, 'kernel_commit': kernel, 'dirty': False}


def schedule(backends, samples, seed):
    """Balance positions within blocks; retain an independently shuffled warmup."""
    rng = random.Random(seed)
    order = list(backends)
    if not order:
        return []
    rng.shuffle(order)
    rounds = [list(order)]
    while len(rounds) <= samples:
        rng.shuffle(order)
        rotations = list(range(len(order)))
        rng.shuffle(rotations)
        rounds.extend(order[i:] + order[:i] for i in rotations)
    return [{'backend': backend, 'round': round_index, 'warmup': round_index == 0,
             'position': position}
            for round_index, order in enumerate(rounds[:samples + 1])
            for position, backend in enumerate(order)]


def measure_case(record, case, args, seed):
    active = [b for b, w in record['workers'].items() if w.get('status') != 'excluded']
    record['schedule'] = schedule(active, args.samples, seed)
    record['validation'] = 'passed'
    anchors = {}
    stopped = set()
    for sequence, entry in enumerate(record['schedule']):
        backend = entry['backend']
        info = record['workers'][backend]
        if backend in stopped:
            sample = {'status': 'not_run', 'reason': 'earlier sample failed for this backend'}
        else:
            sample = worker(info['command'], case, args.timeout, args.address_space_mib, args.cpu)
        sample.update(entry, sequence=sequence)
        info['samples'].append(sample)
        if sample['status'] != 'completed':
            record['validation'] = 'failed'
            stopped.add(backend)
            continue
        try:
            # Cross-backend agreement plus stable backend-specific coverage/bases.
            for anchor in (next(iter(anchors.values()), sample), anchors.get(backend, sample)):
                compare_samples(anchor, sample, case)
            anchors.setdefault(backend, sample)
        except ValueError as error:
            record['validation'] = 'failed'
            sample['comparison_error'] = str(error)
            stopped.add(backend)
    rows = []
    for backend in active:
        completed = [s for s in record['workers'][backend]['samples']
                     if s['status'] == 'completed' and not s['warmup'] and 'comparison_error' not in s]
        # Never publish statistics for incomplete or mismatched cases.
        valid = completed if record['validation'] == 'passed' else []
        rows.append({'case': case.name, 'backend': backend, 'samples': len(completed),
                     'planned_samples': args.samples, 'validation': record['validation'],
                     'timing_scope': timing_scope(case, backend),
                     'median_ms': statistics.median(s['elapsed_ms'] for s in valid) if valid else None,
                     'min_ms': min((s['elapsed_ms'] for s in valid), default=None),
                     'max_ms': max((s['elapsed_ms'] for s in valid), default=None),
                     'max_peak_rss_kib': max((s['peak_rss_kib'] for s in valid), default=None),
                     'max_hwm_growth_kib': max((s['peak_rss_kib'] - s['hwm_before_kib'] for s in valid), default=None),
                     'phase_medians_ms': {name: statistics.median(s['phases_ms'][i] for s in valid) if valid else None
                                          for i, name in enumerate(PHASES)}})
    return rows


def run(args):
    if sys.platform != 'linux':
        raise ValueError('Linux is required for per-process VmHWM and RLIMIT_AS')
    if args.samples < 1 or not math.isfinite(args.timeout) or args.timeout <= 0 or args.address_space_mib < 128:
        raise ValueError('positive sample/timeout and at least 128 MiB address space required')
    identity = provenance(args.kernel_revision)
    affinity = sorted(os.sched_getaffinity(0))
    if args.cpu is not None and args.cpu not in affinity:
        raise ValueError('--cpu must belong to the current allowed affinity')
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    fixture_dir = output / 'fixtures'
    fixture_dir.mkdir()
    started = datetime.now(timezone.utc).isoformat()
    initial_hash = source_hash()
    metadata = build(args, output)
    metadata.update({**identity, 'protocol_id': PROTOCOL, 'order_seed': args.order_seed,
                     'allowed_cpus': affinity, 'worker_cpus': [args.cpu] if args.cpu is not None else affinity,
                     'command': [sys.executable, *sys.argv], 'quick': args.quick,
                     'frequency_and_load_control': 'not controlled by this harness',
                     'started_utc': started, 'source_sha256': initial_hash, 'platform': platform.platform(), 'machine': platform.machine(),
                     'processor': platform.processor(), 'cpuinfo': Path('/proc/cpuinfo').read_text().split('\n\n')[0],
                     'samples': args.samples, 'warmups': 1, 'phases': PHASES, 'timeout_seconds': args.timeout,
                     'address_space_mib': args.address_space_mib, 'RUSTFLAGS': os.environ.get('RUSTFLAGS'),
                     'protocol': __doc__, 'parallel_workers': False})
    (output / 'environment.json').write_text(json.dumps(metadata, indent=2) + '\n')
    records = []
    rows = []
    for case_index, case in enumerate(cases(args.quick)):
        path = fixture_dir / f'{case.name}.txt'
        case.fixture.write(path)
        if case.coordinates is not None:
            Path(str(path) + '.points').write_text(' '.join(map(str, case.coordinates)) + '\n')
        record = {'name': case.name, 'n': case.fixture.n, 'q': case.fixture.q, 'field': case.fixture.characteristic,
                  'cutoff': case.fixture.cutoff, 'path': case.path, 'layout': case.layout, 'epsilon': case.epsilon,
                  'representatives': case.representatives, 'fixture_sha256': sha256(path), 'workers': {}}
        if case.coordinates is not None:
            record['points_sha256'] = sha256(Path(str(path) + '.points'))
        records.append(record)
        for backend in ('cocycle', 'gudhi', 'ripser'):
            reason = exclusion(case, backend)
            if reason:
                record['workers'][backend] = {'status': 'excluded', 'reason': reason}
                continue
            command = [str(output / 'build' / backend), str(path), case.path, case.layout,
                       str(case.epsilon), 'yes' if case.representatives else 'no']
            record['workers'][backend] = {'command': command, 'samples': []}
        record['order_seed'] = args.order_seed + case_index
        rows.extend(measure_case(record, case, args, record['order_seed']))
        (output / 'results.json').write_text(json.dumps(records, indent=2) + '\n')
        (output / 'measurements.json').write_text(json.dumps(rows, indent=2) + '\n')
        print(case.name, record['validation'], flush=True)
    try:
        unchanged = source_hash() == initial_hash and provenance(args.kernel_revision) == identity
    except (ValueError, subprocess.CalledProcessError):
        unchanged = False
    summary = {**identity, 'protocol_id': PROTOCOL, 'sources_unchanged': unchanged,
               'status': 'passed' if unchanged and all(r['validation'] == 'passed' for r in records) else 'failed',
               'finished_utc': datetime.now(timezone.utc).isoformat(),
               'cases': len(records), 'worker_measurements': len(rows), 'samples_per_worker': args.samples,
               'excluded_workers': sum(w.get('status') == 'excluded' for r in records for w in r['workers'].values()),
               'source_sha256': initial_hash, 'failed_cases': [r['name'] for r in records if r['validation'] != 'passed']}
    (output / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n')
    print(json.dumps(summary))
    if summary['status'] != 'passed':
        raise SystemExit(1)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--quick', action='store_true')
    parser.add_argument('--samples', type=int, default=12)
    parser.add_argument('--order-seed', type=int, default=0)
    parser.add_argument('--cpu', type=int, help='pin every worker to this allowed Linux CPU')
    parser.add_argument('--kernel-revision', default='HEAD', help='commit whose Cargo manifests and src are measured')
    parser.add_argument('--timeout', type=float, default=30.)
    parser.add_argument('--address-space-mib', type=int, default=2048)
    parser.add_argument('--gudhi-source', type=Path, default=REPO / 'target/native-sources/gudhi')
    parser.add_argument('--ripser-source', type=Path, default=REPO / 'target/native-sources/ripser')
    parser.add_argument('--boost-include', type=Path)
    parser.add_argument('--cxx', default='g++')
    run(parser.parse_args())


if __name__ == '__main__':
    main()
