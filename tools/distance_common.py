"""Private distance fixture, build, oracle and process helpers (standard library)."""

import hashlib
import itertools
import json
import math
import os
from pathlib import Path
import platform
import random
import struct
import subprocess
import sys
from fractions import Fraction
from functools import lru_cache

ROOT = Path(__file__).resolve().parents[1]
WORKERS = ROOT / 'benches/distances'
PINS = json.loads((WORKERS / 'sources.json').read_text())
PROTOCOL = 'cocycle-distance-v1'
MAGIC = b'COCDST1\0'
METRICS = ('bottleneck', 'w1', 'w2')


def save(path, value):
    Path(path).write_text(json.dumps(value, indent=2, allow_nan=False) + '\n', encoding='utf-8')


def sha256(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def write_fixture(path, first, second):
    """Two lengths followed by interleaved f64 pairs; no per-worker quantization."""
    with Path(path).open('wb') as file:
        file.write(MAGIC + struct.pack('<QQ', len(first), len(second)))
        for birth, death in itertools.chain(first, second):
            file.write(struct.pack('<dd', birth, death))


def read_fixture(path):
    data = Path(path).read_bytes()
    if len(data) < 24 or data[:8] != MAGIC:
        raise ValueError('expected COCDST1 distance fixture')
    n, m = struct.unpack_from('<QQ', data, 8)
    if len(data) != 24 + 16 * (n + m):
        raise ValueError('invalid fixture length')
    points = list(struct.iter_unpack('<dd', data[24:]))
    return points[:n], points[n:]


def clean_points(points):
    """The common raw contract: finite birth, ordered death or positive infinity."""
    finite, essential = [], []
    for birth, death in points:
        if not math.isfinite(birth) or math.isnan(death) or death < birth:
            raise ValueError('invalid endpoint')
        if death == math.inf:
            essential.append(Fraction(birth))
        elif birth != death:
            finite.append((Fraction(birth), Fraction(death)))
    return finite, sorted(essential)


def tiny_oracle(first, second, metric):
    """Enumerate every partial injection, using exact rational costs before sqrt.

    This does not share threshold search, graph construction, matching or routing
    code with either measured implementation. Limited deliberately to tiny input.
    """
    if metric not in METRICS:
        raise ValueError('unknown metric')
    left, essential_left = clean_points(first)
    right, essential_right = clean_points(second)
    if max(len(left), len(right)) > 8:
        raise ValueError('tiny oracle accepts at most eight finite points per side')
    if len(essential_left) != len(essential_right):
        return math.inf
    if len(right) > len(left):
        left, right = right, left
    combine = max if metric == 'bottleneck' else lambda a, b: a + b

    def diagonal(point):
        persistence = point[1] - point[0]
        return persistence * persistence / 2 if metric == 'w2' else persistence / 2

    def pair(a, b):
        dx, dy = abs(a[0] - b[0]), abs(a[1] - b[1])
        return dx * dx + dy * dy if metric == 'w2' else max(dx, dy)

    @lru_cache(None)
    def visit(index, used):
        if index == len(left):
            value = Fraction(0)
            for column, point in enumerate(right):
                if not used & (1 << column):
                    value = combine(value, diagonal(point))
            return value
        value = combine(diagonal(left[index]), visit(index + 1, used))
        for column, point in enumerate(right):
            if not used & (1 << column):
                value = min(value, combine(pair(left[index], point),
                                           visit(index + 1, used | (1 << column))))
        return value

    value = visit(0, 0)
    for a, b in zip(essential_left, essential_right):
        cost = abs(a - b)
        value = combine(value, cost * cost if metric == 'w2' else cost)
    return math.sqrt(float(value)) if metric == 'w2' else float(value)


def number(value):
    return {'value': None, 'value_kind': 'infinite'} if math.isinf(value) else {
        'value': value, 'value_kind': 'finite'}


def unpack_number(record):
    kind, value = record.get('value_kind'), record.get('value')
    if kind == 'infinite' and value is None:
        return math.inf
    if kind != 'finite' or isinstance(value, bool) or not isinstance(value, (int, float)):
        raise ValueError('invalid distance scalar encoding')
    if not math.isfinite(value) or value < 0:
        raise ValueError('distance must be finite and nonnegative')
    return value


def tolerance(case, metric, expected):
    if case.get('exact_grid') and metric in ('bottleneck', 'w1'):
        return 0.0
    costs = [abs(death * 0.5 - birth * 0.5) for birth, death in case['first'] + case['second']
             if math.isfinite(death)]
    for coordinate in (0, 1):
        left = [point[coordinate] for point in case['first'] if math.isfinite(point[coordinate])]
        right = [point[coordinate] for point in case['second'] if math.isfinite(point[coordinate])]
        if left and right:
            costs.extend((abs(max(left) - min(right)), abs(max(right) - min(left))))
    # Fixed before measurement: roundoff allowance scales with input magnitude
    # and the maximum number of terms, not with an observed mismatch.
    scale = max([abs(expected), *costs], default=0.0)
    return 64 * sys.float_info.epsilon * (len(case['first']) + len(case['second']) + 1) * scale


def agrees(actual, expected, case, metric):
    if math.isinf(expected):
        return math.isinf(actual)
    return math.isfinite(actual) and abs(actual - expected) <= tolerance(case, metric, expected)


def correctness_cases(quick=False):
    cases = [
        ('empty', [], []),
        ('diagonal_cost', [(0.0, 2.0)], []),
        ('near_diagonal', [(1.0, 1.0 + 2 ** -20)], [(1.0, 1.0 + 2 ** -19)]),
        ('duplicates', [(0.0, 2.0)] * 3, [(0.0, 2.0)]),
        ('diagonal_points', [(0.0, 0.0), (1.0, 3.0)], [(1.0, 3.0), (5.0, 5.0)]),
        ('negative_scales', [(-4.0, -2.0), (-2.0, 1.0)], [(-3.0, 0.0)]),
        ('essential_equal', [(0.0, math.inf), (0.0, 2.0)], [(1.0, math.inf), (1.0, 3.0)]),
        ('essential_unequal', [(0.0, math.inf)], []),
        ('essential_repeated', [(0.0, math.inf)] * 2, [(1.0, math.inf), (2.0, math.inf)]),
        ('unequal_size', [(0.0, 3.0), (2.0, 4.0), (4.0, 8.0)], [(1.0, 4.0)]),
        ('threshold_ties', [(0.0, 2.0), (2.0, 4.0)], [(1.0, 3.0), (3.0, 5.0)]),
        ('large_finite', [(2.0 ** 400, 2.0 ** 401)], [(2.0 ** 399, 2.0 ** 400)]),
        ('small_finite', [(0.0, 2.0 ** -400)], [(0.0, 2.0 ** -399)]),
        ('adjacent_float_diagonal', [(1.0, math.nextafter(1.0, math.inf))], []),
    ]
    rng = random.Random(0)
    for index in range(4 if quick else 24):
        diagrams = []
        for _ in range(2):
            points = []
            for _ in range(rng.randrange(0, 6)):
                birth = rng.randrange(-16, 16) / 4
                points.append((birth, birth + rng.randrange(1, 17) / 4))
            diagrams.append(points)
        cases.append((f'random-{index}', *diagrams))
    return [{'name': name, 'first': first, 'second': second,
             'exact_grid': name not in ('large_finite', 'small_finite')}
            for name, first, second in cases]


def source_hash():
    paths = [ROOT / 'Cargo.toml', ROOT / 'Cargo.lock', *sorted((ROOT / 'src').rglob('*.rs')),
             *sorted(WORKERS.glob('*')), *(ROOT / 'tools' / name for name in
                ('distance_common.py', 'compare_distances.py', 'benchmark_distances.py'))]
    digest = hashlib.sha256()
    for path in paths:
        if path.is_file():
            digest.update(path.relative_to(ROOT).as_posix().encode() + b'\0' + path.read_bytes())
    return digest.hexdigest()


def identity():
    def git(*args):
        return subprocess.check_output(['git', *args], cwd=ROOT, text=True).strip()
    return {'commit': git('rev-parse', 'HEAD'),
            'dirty': bool(git('status', '--porcelain', '--untracked-files=normal')),
            'source_sha256': source_hash()}


def build_workers(output, topp_source, cxx='c++', cargo='cargo', rustc='rustc'):
    """Build into a fresh directory; read but never modify the external checkout."""
    directory = Path(output) / 'build'
    directory.mkdir()
    topp_source = Path(topp_source).resolve()
    revision = subprocess.check_output(['git', '-C', str(topp_source), 'rev-parse', 'HEAD'], text=True).strip()
    dirty = subprocess.check_output(['git', '-C', str(topp_source), 'status', '--porcelain',
                                     '--untracked-files=all'], text=True)
    if revision != PINS['topp']['revision'] or dirty:
        raise ValueError('Topp requires the pinned clean source checkout; never reset the user checkout')
    before = source_hash()
    commands = []

    def run(command):
        commands.append([str(part) for part in command])
        with (directory / 'build.log').open('a', encoding='utf-8') as log:
            log.write(json.dumps(commands[-1]) + '\n')
            log.flush()
            subprocess.run(commands[-1], cwd=ROOT, check=True, stdout=log, stderr=subprocess.STDOUT)

    suffix = '.exe' if os.name == 'nt' else ''
    target = directory / 'cargo'
    run([cargo, 'build', '--release', '--locked', '--offline', '--lib', '--target-dir', target])
    rust_binary = directory / ('cocycle' + suffix)
    run([rustc, '--edition=2024', '-C', 'opt-level=3', '-D', 'warnings', WORKERS / 'cocycle.rs',
         '--extern', f'cocycle={target}/release/libcocycle.rlib', '-L',
         f'dependency={target}/release/deps', '-o', rust_binary])
    topp_binary = directory / ('topp' + suffix)
    sources = [topp_source / 'src' / name for name in
               ('bottleneck_core.cpp', 'geometric_backend.cpp', 'wasserstein.cpp')]
    run([cxx, '-std=c++20', '-O3', '-DNDEBUG', '-pthread', '-I', topp_source / 'include',
         *sources, WORKERS / 'topp.cpp', '-o', topp_binary])
    if source_hash() != before:
        raise ValueError('distance sources changed while compiling; use a new run')
    metadata = {'pins': PINS, 'commands': commands, 'topp_revision': revision,
                'topp_source_sha256': {str(p.relative_to(topp_source)): sha256(p)
                                       for directory in ('src', 'include')
                                       for p in sorted((topp_source / directory).rglob('*'))
                                       if p.is_file() and p.suffix in ('.hpp', '.cpp')},
                'binaries_sha256': {'cocycle': sha256(rust_binary), 'topp': sha256(topp_binary)},
                'rustc': subprocess.check_output([rustc, '-Vv'], text=True),
                'cxx': subprocess.check_output([cxx, '--version'], text=True),
                'avx2': 'not enabled; pinned Topp portable scalar build',
                'rustflags': os.environ.get('RUSTFLAGS'),
                'cxx_arithmetic': 'Topp compiler-dependent long double in weighted matching; Rust f64',
                'source_sha256': before}
    save(directory / 'build.json', metadata)
    return {'cocycle': [str(rust_binary)], 'topp': [str(topp_binary)]}, metadata


def invoke(command, fixture, metric, variant='baseline', timeout=60, memory_mib=None, cpu=None):
    """One invocation is one fresh process. Preserve failures as data."""
    kwargs = {}
    if memory_mib is not None or cpu is not None:
        if platform.system() != 'Linux':
            raise ValueError('resource-limited distance measurements require Linux')
        import resource

        def limits():
            if memory_mib is not None:
                limit = memory_mib * 1024 * 1024
                resource.setrlimit(resource.RLIMIT_AS, (limit, limit))
            if cpu is not None:
                os.sched_setaffinity(0, {cpu})
        kwargs['preexec_fn'] = limits
    args = [*command, str(fixture), metric, variant]
    try:
        result = subprocess.run(args, capture_output=True, text=True, timeout=timeout, **kwargs)
    except subprocess.TimeoutExpired:
        return {'status': 'timeout', 'timeout_seconds': timeout, 'command': args}
    except OSError as error:
        return {'status': 'process_error', 'error': str(error), 'command': args}
    retained = {'command': args, 'returncode': result.returncode,
                'stdout': result.stdout, 'stderr': result.stderr}
    if result.returncode:
        return {'status': 'process_error', **retained}
    try:
        value = json.loads(result.stdout)
        if value['protocol_id'] != PROTOCOL or value['metric'] != metric or value['variant'] != variant:
            raise ValueError('worker identity mismatch')
        if value['status'] == 'completed':
            unpack_number(value)
            elapsed = value.get('elapsed_ms')
            if isinstance(elapsed, bool) or not isinstance(elapsed, (int, float)) or not math.isfinite(elapsed) or elapsed < 0:
                raise ValueError('invalid elapsed time')
        elif value['status'] not in ('rejected', 'unsupported', 'unavailable'):
            raise ValueError('invalid worker status')
        return {**value, **retained}
    except (KeyError, ValueError, TypeError) as error:
        return {'status': 'protocol_error', 'error': str(error), **retained}


def add_build_arguments(parser):
    parser.add_argument('--output', type=Path, required=True, help='new ignored artifact directory')
    parser.add_argument('--topp-source', type=Path, default=ROOT / 'target/native-sources/topp')
    parser.add_argument('--cxx', default='c++')
    parser.add_argument('--cargo', default='cargo')
    parser.add_argument('--rustc', default='rustc')
    parser.add_argument('--timeout', type=float, default=60)
    parser.add_argument('--gudhi-python', default=sys.executable)


def build_from_args(args):
    if args.timeout <= 0:
        raise ValueError('timeout must be positive')
    args.output = args.output.resolve()
    args.output.mkdir(parents=True, exist_ok=False)
    return build_workers(args.output, args.topp_source, args.cxx, args.cargo, args.rustc)
