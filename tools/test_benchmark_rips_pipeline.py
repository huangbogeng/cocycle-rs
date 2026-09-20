"""Protocol failures must not turn into successful performance comparisons."""
import copy
from dataclasses import replace
import subprocess
import unittest
from unittest.mock import patch

from benchmark_rips_pipeline import (Case, cases, compare_samples, exclusion,
                                    instrument_sparse, timing_scope, validate_output, worker)
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


class TimingScopeTests(unittest.TestCase):
    def test_no_speed_ratio_for_unmatched_output_or_input_contracts(self):
        fixture = Fixture('pair', 'dense', 2, 1, None, [1.])
        for case in (Case(fixture, 'expanded'), Case(fixture, 'dense', 'upper'),
                     Case(fixture, 'threshold', representatives=True), Case(fixture, 'points')):
            self.assertEqual(timing_scope(case, 'ripser'), 'correctness_reference_only')
            self.assertEqual(timing_scope(case, 'cocycle'), 'workflow')
        self.assertEqual(timing_scope(Case(fixture, 'expanded'), 'gudhi'), 'workflow')
        self.assertEqual(timing_scope(Case(fixture, 'dense'), 'ripser'), 'workflow')
