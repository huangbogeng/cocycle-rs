"""Guard native instrumentation scope and reference fixture semantics."""
import unittest

from compare_sparse_rips import cases, independent_simplices, instrument_sparse


class SparseReferenceTests(unittest.TestCase):
    def test_instrumentation_changes_only_one_sampling_callee(self):
        original = 'before subsampling::choose_n_farthest_points_metric(args); after'
        self.assertEqual(instrument_sparse(original), 'before ::sparse_reference::sample(args); after')
        for source in ('no hook', original + original):
            with self.assertRaises(ValueError):
                instrument_sparse(source)

    def test_blocker_fixture_retains_edges_but_not_triangle(self):
        case, epsilon, minimum, start, dimension = next(c for c in cases() if c[0].name == 'blocker_dim_7')
        self.assertEqual(start, 0)
        simplices, coverage = independent_simplices(case, epsilon, minimum,
                                                   [0, 4, 3, 5, 6, 1, 2, 7],
                                                   [None, 76., 73., 48., 32., 24., 20., 18.], dimension)
        topology = {tuple(v): value for v, value in simplices}
        self.assertIsNone(coverage)
        for edge in ((1, 2), (1, 3), (2, 3)):
            self.assertIn(edge, topology)
        self.assertEqual(max(topology[e] for e in ((1, 2), (1, 3), (2, 3))), 178.)
        self.assertNotIn((1, 2, 3), topology)

    def test_fixture_names_are_unique_and_include_dimension_zero_and_all_fields(self):
        fixtures = list(cases())
        names = [c[0].name for c in fixtures]
        self.assertEqual(len(names), len(set(names)))
        self.assertIn(0, {c[4] for c in fixtures})
        self.assertEqual({c[0].characteristic for c in fixtures}, {2, 3, 5, 251, 46337})
