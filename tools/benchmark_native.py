"""Native Rust/GUDHI C++/upstream Ripser C++ benchmarks; Python only orchestrates.

One fresh process and one measured native call per sample, without warmups.
No Python TDA package is imported. See benches/protocol.md for interpretation.
"""

import argparse
import csv
from datetime import datetime, timezone
import json
import math
import os
from pathlib import Path
import platform
import random
import statistics
import subprocess
import sys

import build_native
from benchmark_inputs import (semantic_cases, performance_cases, float32_case,
                              make_case, case_specs, bipartite_diagram)

BACKENDS = ('cocycle', 'gudhi_cpp', 'gudhi_collapse_cpp', 'ripser_cpp')
REPO = build_native.REPO


def save(path, data):
    path.write_text(json.dumps(data, indent=2, allow_nan=False) + '\n')


def compare(left, right):
    """No tolerance: every supplied filtration value is exactly shared."""
    if left['coverage'] != right['coverage']:
        raise ValueError('coverage differs')
    if sorted(left['intervals']) != sorted(right['intervals']):
        raise ValueError('diagram multisets differ (dimensions, endpoints or multiplicity)')


def expected(case):
    """Hand-derived semantic cases, independent of every persistence worker."""
    diameter = max(case.values, default=0.)
    complete = case.cutoff is None or case.cutoff >= diameter
    survivor = [0, 0., 'E' if complete else 'C', 0. if complete else case.cutoff]
    if case.name.startswith('check_empty_'):
        bars = []
    elif case.name.startswith(('check_singleton_', 'check_duplicates_')):
        bars = [survivor]
    elif case.name.startswith('check_pair_'):
        bars = [[0, 0., 'F', 2.], survivor]
    elif case.name.startswith('check_tetrahedron_'):
        bars = [[0, 0., 'F', 1.]] * 3 + [survivor]
    elif case.name.startswith('check_square_'):
        if case.cutoff is not None and case.cutoff < 1:
            bars = [survivor] * 4
        else:
            bars = [[0, 0., 'F', 1.]] * 3 + [survivor]
            if case.q == 1:
                bars += [[1, 1., 'F' if complete else 'C', diameter if complete else case.cutoff]]
    elif case.name.startswith('bipartite_'):
        return bipartite_diagram(case.n, case.cutoff)
    else:
        return None
    return {'coverage': ['complete', None] if complete else ['through', case.cutoff], 'intervals': bars}


def valid_worker(data, case, backend):
    """Reject malformed output before admitting samples or comparing diagrams."""
    if not isinstance(data, dict):
        raise ValueError('worker output is not an object')
    if data.get('status') == 'unsupported':
        if backend != 'ripser_cpp' or case.n >= 2 or not data.get('reason'):
            raise ValueError('unexpected exclusion')
        return
    if data.get('status') != 'completed':
        raise ValueError('missing completed status')
    elapsed = data['elapsed_ms']
    if type(elapsed) not in (int, float) or not math.isfinite(elapsed) or elapsed <= 0:
        raise ValueError('invalid native elapsed time')
    complete = case.cutoff is None or case.cutoff >= max(case.values, default=0.)
    coverage = ['complete', None] if complete else ['through', case.cutoff]
    if data['coverage'] != coverage:
        raise ValueError('worker coverage does not match input')
    for bar in data['intervals']:
        if not isinstance(bar, list) or len(bar) != 4:
            raise ValueError('invalid interval shape')
        dim, birth, kind, end = bar
        if type(dim) is not int or dim < 0 or dim > case.q:
            raise ValueError('unexpected dimension')
        if any(type(v) not in (int, float) or not math.isfinite(v) or v < 0 for v in (birth, end)):
            raise ValueError('invalid interval scale')
        if kind not in ('F', 'E', 'C') or (kind == 'F' and end <= birth):
            raise ValueError('invalid endpoint or zero-length interval')
        if kind == 'E' and (not complete or end != 0):
            raise ValueError('incorrect essential endpoint')
        if kind == 'C' and (complete or end != case.cutoff or birth > end):
            raise ValueError('incorrect censored endpoint')
        if not complete and (birth > case.cutoff or (kind == 'F' and end > case.cutoff)):
            raise ValueError('interval beyond known coverage')
    for key in ('rss_before_kib', 'hwm_before_kib', 'peak_rss_kib'):
        if type(data[key]) is not int or data[key] < 0:
            raise ValueError('missing Linux memory measurement')
    if data['peak_rss_kib'] < data['hwm_before_kib']:
        raise ValueError('high-water mark went backwards')


def worker(command, case, backend, timeout, address_space_mib, env):
    import resource

    def limits():
        limit = address_space_mib * 1024 * 1024
        resource.setrlimit(resource.RLIMIT_AS, (limit, limit))

    try:
        result = subprocess.run(command, text=True, capture_output=True, timeout=timeout,
                                env=env, preexec_fn=limits)
    except subprocess.TimeoutExpired:
        return {'status': 'timeout', 'timeout_seconds': timeout}
    except (OSError, subprocess.SubprocessError) as error:
        return {'status': 'process_error', 'error': repr(error)}
    if result.returncode:
        return {'status': 'process_error', 'returncode': result.returncode,
                'stderr': result.stderr[-8000:], 'stdout': result.stdout[-2000:]}
    try:
        data = json.loads(result.stdout)
        valid_worker(data, case, backend)
        return data
    except (ValueError, KeyError, TypeError) as error:
        return {'status': 'protocol_error', 'error': str(error), 'stdout': result.stdout[-2000:]}


def validate(record, analytic=None):
    complete = [(name, result) for name, result in record['results'].items()
                if result['status'] == 'completed']
    record['validation'] = 'unverified'
    record['comparisons'] = []
    if not complete:
        return
    try:
        anchor_name, anchor = complete[0]
        if analytic is not None:
            compare(anchor, analytic)
            record['comparisons'].append([anchor_name, 'analytic'])
        for name, result in complete[1:]:
            compare(anchor, result)
            record['comparisons'].append([anchor_name, name])
        if record['comparisons']:
            record['validation'] = 'passed'
    except ValueError as error:
        record['validation'] = 'mismatch'
        record['error'] = str(error)


def summarize(output, records):
    rows = []
    for record in records:
        for backend, result in record['results'].items():
            verified = record['validation'] == 'passed' and result['status'] == 'completed'
            samples = result.get('samples', [])
            rows.append({
                'case': record['case'], 'kind': record['kind'], 'n': record['n'], 'backend': backend,
                'status': result['status'], 'validation': record['validation'],
                'median_ms': statistics.median(s['elapsed_ms'] for s in samples) if verified else None,
                'min_ms': min(s['elapsed_ms'] for s in samples) if verified else None,
                'max_ms': max(s['elapsed_ms'] for s in samples) if verified else None,
                'max_peak_rss_kib': max(s['peak_rss_kib'] for s in samples) if verified else None,
                'max_hwm_growth_kib': max(s['peak_rss_kib'] - s['hwm_before_kib'] for s in samples) if verified else None,
                'intervals': len(result['intervals']) if result.get('intervals') is not None else None,
                'reason': result.get('reason', result.get('error')),
            })
    if rows:
        with output.open('w', newline='') as stream:
            writer = csv.DictWriter(stream, fieldnames=rows[0].keys())
            writer.writeheader()
            writer.writerows(rows)


def run_cases(output, directory, cases, args, env):
    records = []
    rng = random.Random(20260920)
    for kind, case in cases:
        fixture = output / 'fixtures' / f'{case.name}.bin'
        case.write(fixture)
        commands = {
            'cocycle': [str(directory / 'cocycle'), str(fixture)],
            'gudhi_cpp': [str(directory / 'gudhi'), str(fixture), 'direct'],
            'gudhi_collapse_cpp': [str(directory / 'gudhi'), str(fixture), 'collapse'],
            'ripser_cpp': [str(directory / 'ripser'), str(fixture)],
        }
        record = {'case': case.name, 'kind': kind, 'n': case.n, 'homology_max': case.q,
                  'cutoff': case.cutoff, 'fixture_sha256': build_native.sha256(fixture),
                  'results': {}, 'sample_order': [], 'validation': 'unverified'}
        records.append(record)
        print(f'{kind}: {case.name}', flush=True)
        for sample_index in range(args.samples if kind == 'benchmark' else 1):
            order = list(BACKENDS)
            rng.shuffle(order)
            record['sample_order'].append(order)
            for backend in order:
                prior = record['results'].get(backend)
                if prior is not None and prior['status'] != 'completed':
                    continue
                result = worker(commands[backend], case, backend, args.timeout, args.address_space_mib, env)
                if result['status'] != 'completed':
                    if prior:
                        result['partial_result'] = prior
                    record['results'][backend] = result
                else:
                    try:
                        if prior is not None:
                            compare(prior, result)
                        else:
                            prior = {'status': 'completed', 'coverage': result['coverage'],
                                     'intervals': result['intervals'], 'samples': []}
                            record['results'][backend] = prior
                        prior['samples'].append({key: value for key, value in result.items()
                                                 if key not in ('coverage', 'intervals', 'status')})
                    except ValueError as error:
                        record['results'][backend] = {
                            'status': 'protocol_error', 'error': f'repeated native calls disagree: {error}',
                            'partial_result': prior, 'unexpected_diagram': result,
                        }
                save(output / 'results.json', records)
        validate(record, expected(case))
        save(output / 'results.json', records)
        summarize(output / 'summary.csv', records)
    return records


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True, help='new output directory; never overwritten')
    parser.add_argument('--gudhi-source', type=Path, default=REPO / 'target/native-sources/gudhi')
    parser.add_argument('--ripser-source', type=Path, default=REPO / 'target/native-sources/ripser')
    parser.add_argument('--boost-include', type=Path, help='directory containing boost/; otherwise system headers')
    parser.add_argument('--cxx', default='c++')
    parser.add_argument('--suite', choices=('baseline', 'scaling'), default='baseline')
    parser.add_argument('--quick', action='store_true')
    parser.add_argument('--samples', type=int, default=3)
    parser.add_argument('--timeout', type=float, default=60., help='whole process seconds per sample, not native elapsed time')
    parser.add_argument('--address-space-mib', type=int, default=2048)
    parser.add_argument('--cpu', type=int)
    args = parser.parse_args()
    if sys.platform != 'linux':
        parser.error('native measurement currently requires Linux RSS and RLIMIT_AS')
    if args.samples < 1 or args.address_space_mib < 1 or not math.isfinite(args.timeout) or args.timeout <= 0:
        parser.error('sample count, time and address-space limits must be positive')
    if args.cpu is not None:
        if args.cpu not in os.sched_getaffinity(0):
            parser.error('--cpu must be available in the current affinity mask')
        os.sched_setaffinity(0, {args.cpu})
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    (output / 'fixtures').mkdir()
    (output / 'manifest-at-measurement.toml').write_bytes((REPO / 'Cargo.toml').read_bytes())
    threads = {key: '1' for key in ('OMP_NUM_THREADS', 'OPENBLAS_NUM_THREADS', 'MKL_NUM_THREADS')}
    env = dict(os.environ, **threads)
    metadata = {'schema': 'cocycle-native-v1', 'status': 'building',
                'created_utc': datetime.now(timezone.utc).isoformat(), 'backends': BACKENDS,
                'platform': platform.platform(), 'python_controller': sys.version,
                'cpu': next((line.split(':', 1)[1].strip() for line in Path('/proc/cpuinfo').read_text().splitlines()
                             if line.startswith('model name')), platform.processor()),
                'affinity': sorted(os.sched_getaffinity(0)), 'threads': threads,
                'samples': args.samples, 'warmups': 0, 'calls_per_process': 1,
                'suite': args.suite, 'quick': args.quick, 'order_seed': 20260920, 'input_seed': 1729,
                'precision': 'shared float32-exact input; f64 Cocycle/GUDHI and upstream f32 Ripser',
                'timing': 'prepared native input to owned normalized diagram; no startup, I/O or final output destruction',
                'timeout_seconds_per_process': args.timeout, 'address_space_mib': args.address_space_mib,
                'pins': build_native.PINS}
    save(output / 'environment.json', metadata)
    try:
        directory = output / 'build'
        info = build_native.build(directory, args.gudhi_source, args.ripser_source, args.boost_include, args.cxx)
        metadata['source_sha256'] = info['source_sha256']
        metadata['status'] = 'running'
        save(output / 'environment.json', metadata)
        cases = [('semantic', float32_case(case)) for case in semantic_cases()]
        if args.suite == 'baseline':
            cases += [('benchmark', float32_case(case)) for case in performance_cases(args.quick)
                      if case.mode == 'distances']
            cases += [('semantic', make_case('bipartite', 8, cutoff)) for cutoff in (None, 1.)]
        else:
            cases += [('benchmark', make_case(family, n, cutoff)) for family, n, cutoff in case_specs(args.quick)]
        records = run_cases(output, directory, cases, args, env)
        if build_native.source_hash() != metadata['source_sha256']:
            raise ValueError('benchmark sources changed during measurement; discard this run')
        statuses = [result['status'] for record in records for result in record['results'].values()]
        failed = any(status in ('process_error', 'protocol_error') for status in statuses) or any(
            record['validation'] == 'mismatch' for record in records)
        limited = 'timeout' in statuses or any(record['validation'] == 'unverified' for record in records)
        metadata['status'] = ('failed' if failed else 'completed_with_limits' if limited else
                              'passed_with_exclusions' if 'unsupported' in statuses else 'passed')
        metadata['case_count'] = len(records)
        metadata['backend_status_counts'] = {s: statuses.count(s) for s in sorted(set(statuses))}
        metadata['comparisons_passed'] = sum(len(r['comparisons']) for r in records if r['validation'] == 'passed')
        return_code = int(failed or limited)
    except BaseException as error:
        metadata['status'] = 'failed'
        metadata['error'] = repr(error)
        raise
    finally:
        save(output / 'environment.json', metadata)
    print(f'{metadata["status"]}: {metadata["case_count"]} cases; {output}')
    return return_code


if __name__ == '__main__':
    raise SystemExit(main())
