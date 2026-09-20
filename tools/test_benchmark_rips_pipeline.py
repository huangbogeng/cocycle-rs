"""Protocol failures must not turn into successful performance comparisons."""
import copy
from collections import Counter
from dataclasses import replace
from types import SimpleNamespace
import subprocess
import unittest
from unittest.mock import patch

from benchmark_rips_pipeline import (Case, cases, compare_samples, exclusion,
                                    instrument_sparse, measure_case, provenance, schedule,
                                    timing_scope, validate_output, worker)
from compare_rips import Fixture


class PipelineTests(unittest.TestCase):
    def setUp(self):
        self.case = Case(Fixture('pair', 'dense', 2, 0, None, [1.]), 'dense')
        self.output = {'status': 'completed', 'elapsed_ms': 5., 'phases_ms': [1.] * 5,
                       'rss_before_kib': 100, 'hwm_before_kib': 100, 'peak_rss_kib': 200,
                       'intervals': [[0, 0., 1.], [0, 0., None]], 'simplices': None, 'permutation': []}

    def test_invalid_times_memory_and_intervals_are_rejected(self):
        validate_output(self.output, self.case)
        invalid = [('elapsed_ms', 0.), ('elapsed_ms', float('nan')), ('phases_ms', [1.] * 6),
                   ('phases_ms', [10.] * 5), ('phases_ms', [-1.] * 5),
                   ('peak_rss_kib', 90), ('rss_before_kib', None),
                   ('intervals', [[0, 1., 1.]]), ('intervals', [[1, 0., None]]),
                   ('intervals', [[0, 0., float('nan')]])]
        for key, value in invalid:
            data = copy.deepcopy(self.output)
            data[key] = value
            with self.subTest(key=key, value=value), self.assertRaises((ValueError, TypeError)):
                validate_output(data, self.case)
        with self.assertRaises(ValueError):
            validate_output([], self.case)

    def test_multiplicity_sampling_and_expansion_mismatches_fail(self):
        other = copy.deepcopy(self.output)
        other['intervals'].reverse()
        compare_samples(self.output, other, self.case)
        other['intervals'].append([0, 0., None])
        with self.assertRaises(ValueError):
            compare_samples(self.output, other, self.case)
        other = dict(self.output, simplices=4)
        with self.assertRaises(ValueError):
            compare_samples(dict(self.output, simplices=3), other, self.case)
        sparse = replace(self.case, path='approximate')
        with self.assertRaises(ValueError):
            compare_samples(dict(self.output, permutation=[0, 1]), other, sparse)

    def test_source_hook_keeps_real_metric_sampler_and_requires_exact_match(self):
        text = 'choose_n_farthest_points_metric(dist, boost::size(points)), -1, -1, out);\n  Graph graph_;'
        edited = instrument_sparse(text)
        self.assertIn('choose_n_farthest_points_metric', edited)
        self.assertIn('boost::size(points)), -1, 0,', edited)
        self.assertIn('benchmark_permutation', edited)
        for bad in (text + text, 'missing'):
            with self.assertRaises(ValueError):
                instrument_sparse(bad)

    def test_timeouts_process_and_protocol_failures_are_retained(self):
        with patch('subprocess.run', side_effect=subprocess.TimeoutExpired('test', 1)):
            self.assertEqual(worker(['test'], self.case, 1, 128)['status'], 'timeout')
        with patch('subprocess.run', return_value=subprocess.CompletedProcess(['test'], 9, '', 'failure')):
            self.assertEqual(worker(['test'], self.case, 1, 128)['returncode'], 9)
        with patch('subprocess.run', return_value=subprocess.CompletedProcess(['test'], 0, '[]', '')):
            self.assertEqual(worker(['test'], self.case, 1, 128)['status'], 'protocol_error')
        with patch('subprocess.run', side_effect=OSError('missing executable')):
            self.assertEqual(worker(['test'], self.case, 1, 128)['status'], 'process_error')

    def test_fixture_scopes_and_exclusions_are_explicit(self):
        fixtures = list(cases(True))
        self.assertEqual(len(fixtures), len({c.name for c in fixtures}))
        for case in fixtures:
            self.assertEqual(bool(exclusion(case, 'ripser')), case.path.startswith('approximate'))
        self.assertTrue(any(c.representatives for c in fixtures))
        self.assertTrue(any(c.coordinates is not None for c in fixtures))

    def test_schedule_reproducible_and_balanced_with_separate_warmups(self):
        for backends in (['cocycle', 'gudhi'], ['cocycle', 'gudhi', 'ripser']):
            entries = schedule(backends, 12, 42)
            self.assertEqual(entries, schedule(backends, 12, 42))
            self.assertNotEqual(entries, schedule(backends, 12, 99))
            warmups = [e for e in entries if e['warmup']]
            self.assertEqual(set(e['backend'] for e in warmups), set(backends))
            self.assertEqual(entries[:len(backends)], warmups)
            for round_index in range(1, 13):
                group = [e for e in entries if e['round'] == round_index]
                self.assertEqual(set(e['backend'] for e in group), set(backends))
            positions = Counter((e['backend'], e['position']) for e in entries if not e['warmup'])
            self.assertEqual(set(positions.values()), {12 // len(backends)})
        self.assertEqual(len(schedule(['cocycle', 'gudhi', 'ripser'], 1, 42)), 6)

    def test_failed_groups_keep_planned_samples_but_no_statistics(self):
        for failure in ({'status': 'timeout'}, dict(self.output, intervals=[])):
            record = {'workers': {b: {'command': [b], 'samples': []} for b in ('cocycle', 'gudhi')}}
            calls = Counter()

            def execute(command, *args):
                backend = command[0]
                calls[backend] += 1
                return copy.deepcopy(failure if backend == 'gudhi' and calls[backend] == 2 else self.output)

            args = SimpleNamespace(samples=3, timeout=1, address_space_mib=128, cpu=None)
            with patch('benchmark_rips_pipeline.worker', side_effect=execute):
                rows = measure_case(record, self.case, args, 0)
            self.assertEqual(record['validation'], 'failed')
            self.assertTrue(all(r['median_ms'] is None for r in rows))
            self.assertEqual(calls, {'cocycle': 4, 'gudhi': 2})
            self.assertEqual(len(record['workers']['gudhi']['samples']), 4)
            self.assertEqual(record['workers']['gudhi']['samples'][-1]['status'], 'not_run')

    def test_backend_specific_metadata_checked_when_native_is_first(self):
        record = {'workers': {b: {'command': [b], 'samples': []} for b in ('gudhi', 'cocycle')}}
        calls = Counter()

        def execute(command, *args):
            backend = command[0]
            calls[backend] += 1
            output = copy.deepcopy(self.output)
            if backend == 'cocycle':
                output['coverage'] = calls[backend]
            return output

        entries = [{'backend': b, 'round': r, 'warmup': r == 0, 'position': i}
                   for r in range(2) for i, b in enumerate(('gudhi', 'cocycle'))]
        args = SimpleNamespace(samples=1, timeout=1, address_space_mib=128, cpu=None)
        with patch('benchmark_rips_pipeline.worker', side_effect=execute), patch(
                'benchmark_rips_pipeline.schedule', return_value=entries):
            measure_case(record, self.case, args, 0)
        self.assertEqual(record['validation'], 'failed')
        self.assertIn('coverage', record['workers']['cocycle']['samples'][-1]['comparison_error'])

    def test_provenance_rejects_dirty_or_different_kernel(self):
        with patch('subprocess.check_output', return_value=' M src/lib.rs'):
            with self.assertRaisesRegex(ValueError, 'commit or stash'):
                provenance('HEAD')
        with patch('subprocess.check_output', side_effect=['', 'harness', 'kernel', 'diff']):
            with self.assertRaisesRegex(ValueError, 'differs'):
                provenance('HEAD')
        with patch('subprocess.check_output', side_effect=['', 'harness', 'kernel', '']):
            self.assertEqual(provenance('HEAD'), {
                'harness_commit': 'harness', 'kernel_commit': 'kernel', 'dirty': False})



class TimingScopeTests(unittest.TestCase):
    def test_no_speed_ratio_for_unmatched_output_or_input_contracts(self):
        fixture = Fixture('pair', 'dense', 2, 1, None, [1.])
        for case in (Case(fixture, 'expanded'), Case(fixture, 'dense', 'upper'),
                     Case(fixture, 'threshold', representatives=True), Case(fixture, 'points')):
            self.assertEqual(timing_scope(case, 'ripser'), 'correctness_reference_only')
            self.assertEqual(timing_scope(case, 'cocycle'), 'workflow')
        self.assertEqual(timing_scope(Case(fixture, 'expanded'), 'gudhi'), 'workflow')
        self.assertEqual(timing_scope(Case(fixture, 'dense'), 'ripser'), 'workflow')
