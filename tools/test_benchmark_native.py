"""Native comparison failure, precision, and independent-validation contracts."""
import copy
import csv
from pathlib import Path
import struct
import sys
import tempfile
import unittest

import benchmark_native as native
import build_native
from benchmark_inputs import Case, float32_case, semantic_cases


def square():
    case = next(case for case in semantic_cases() if case.name == 'check_square_cycle_h1')
    result = dict(native.expected(case), status='completed', elapsed_ms=1.,
                  rss_before_kib=100, hwm_before_kib=120, peak_rss_kib=130)
    return case, result


class NativeBenchmarkTests(unittest.TestCase):
    def test_fixture_bytes_preserve_shared_quantized_input(self):
        case = float32_case(Case('pair', 2, [.1], cutoff=.3))
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / 'pair.bin'
            case.write(path)
            magic, mode, n, d, q, cutoff, distance = struct.unpack('<8sQQQQdd', path.read_bytes())
        self.assertEqual((magic, mode, n, d, q), (b'COCYCLE1', 0, 2, 2, 1))
        self.assertEqual(distance, struct.unpack('<f', struct.pack('<f', .1))[0])
        self.assertEqual(cutoff, struct.unpack('<f', struct.pack('<f', .3))[0])

    def test_worker_rejects_incorrect_endpoint_semantics_and_dimensions(self):
        case, result = square()
        native.valid_worker(result, case, 'gudhi_cpp')
        changes = [[1, 1., 'E', 0.], [1, 1., 'F', 1.], [1, 1., 'C', 2.],
                   [2, 1., 'C', 1.], [1, float('nan'), 'C', 1.]]
        for bar in changes:
            with self.subTest(bar=bar):
                invalid = dict(result, intervals=[bar])
                with self.assertRaises(ValueError):
                    native.valid_worker(invalid, case, 'gudhi_cpp')
        for elapsed in (float('nan'), float('inf'), -1, 0, True):
            with self.assertRaises(ValueError):
                native.valid_worker(dict(result, elapsed_ms=elapsed), case, 'cocycle')

    def test_only_declared_ripser_small_inputs_can_be_excluded(self):
        empty = Case('empty', 0, [])
        excluded = {'status': 'unsupported', 'reason': 'small input'}
        native.valid_worker(excluded, empty, 'ripser_cpp')
        for backend, case in [('gudhi_cpp', empty), ('ripser_cpp', Case('pair', 2, [1.]))]:
            with self.assertRaises(ValueError):
                native.valid_worker(excluded, case, backend)

    def test_compare_preserves_multiplicity_and_exact_endpoints(self):
        case, result = square()
        duplicate = copy.deepcopy(result)
        duplicate['intervals'].append(result['intervals'][0])
        with self.assertRaises(ValueError):
            native.compare(result, duplicate)
        altered = copy.deepcopy(result)
        altered['intervals'][0][3] += 1e-12
        with self.assertRaises(ValueError):
            native.compare(result, altered)
        reordered = dict(result, intervals=list(reversed(result['intervals'])))
        native.compare(result, reordered)

    def test_external_results_are_checked_when_cocycle_times_out(self):
        _, result = square()
        record = {'results': {'cocycle': {'status': 'timeout'}, 'gudhi_cpp': result,
                              'ripser_cpp': copy.deepcopy(result)}}
        native.validate(record)
        self.assertEqual(record['validation'], 'passed')
        record['results']['ripser_cpp']['intervals'].pop()
        native.validate(record)
        self.assertEqual(record['validation'], 'mismatch')

    def test_unverified_or_failed_results_get_no_summary_measurement(self):
        case, result = square()
        result['samples'] = [dict(elapsed_ms=1., hwm_before_kib=120, peak_rss_kib=130)]
        record = {'case': case.name, 'kind': 'benchmark', 'n': 4,
                  'results': {'cocycle': result, 'ripser_cpp': {'status': 'timeout'}}}
        native.validate(record)
        self.assertEqual(record['validation'], 'unverified')
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / 'summary.csv'
            native.summarize(path, [record])
            with path.open() as stream:
                rows = list(csv.DictReader(stream))
        self.assertTrue(all(row['median_ms'] == '' for row in rows))
        self.assertEqual(rows[1]['status'], 'timeout')

    @unittest.skipUnless(sys.platform == "linux", "native workers require Linux process limits")
    def test_invalid_worker_output_is_a_protocol_failure(self):
        case, _ = square()
        for payload in ('{}', '{"status":"completed","elapsed_ms":NaN}', 'not JSON'):
            result = native.worker([sys.executable, '-c', f'print({payload!r})'],
                                   case, 'gudhi_cpp', 5, 256, None)
            self.assertEqual(result['status'], 'protocol_error')

    def test_output_adapter_refuses_unmatched_source(self):
        with self.assertRaises(ValueError):
            build_native.instrument_ripser('int main() { return 0; }')

    @unittest.skipUnless(sys.platform == "linux", "native workers require Linux process limits")
    def test_worker_launch_failure_and_timeout_remain_explicit(self):
        case, _ = square()
        with tempfile.TemporaryDirectory() as tmp:
            missing = native.worker([str(Path(tmp) / 'missing')], case, 'gudhi_cpp', 1, 256, None)
        self.assertEqual(missing['status'], 'process_error')
        timeout = native.worker([sys.executable, '-c', 'import time; time.sleep(1)'],
                                case, 'cocycle', .01, 256, None)
        self.assertEqual(timeout['status'], 'timeout')
        self.assertNotIn('elapsed_ms', timeout)


if __name__ == '__main__':
    unittest.main()
