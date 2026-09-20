"""Regressions for native comparison semantics, not external-library mocks."""
import tempfile
import unittest
from pathlib import Path
from compare_rips import Fixture, compare, fixtures, reference_exclusion


class ComparisonTests(unittest.TestCase):
    def test_truncated_original_range_is_not_largest_retained_edge(self):
        case = Fixture('range', 'dense', 3, 1, 1.5, [1., 2., 1.])
        self.assertEqual(case.expected_edges(), [[0, 1, 1.], [1, 2, 1.]])
        self.assertEqual(case.coverage(), 1.5)
        result = {'status': 'ok', 'characteristic': 2, 'coverage': 1., 'edges': case.expected_edges(), 'intervals': []}
        with self.assertRaisesRegex(ValueError, 'coverage'):
            compare(case, result, result)

    def test_multiplicity_and_exact_values_are_not_erased(self):
        case = Fixture('pair', 'flag', 2, 0, None, [])
        actual = {'status': 'ok', 'characteristic': 2, 'coverage': None, 'edges': [], 'intervals': [[0, 0., None]] * 2}
        compare(case, actual, actual)
        with self.assertRaisesRegex(ValueError, 'multiset'):
            compare(case, actual, {**actual, 'intervals': [[0, 0., None]]})
        with self.assertRaisesRegex(ValueError, 'cannot count'):
            compare(case, actual, {'status': 'unsupported'})

    def test_expansion_checks_faces_and_values(self):
        case = Fixture('triangle', 'dense', 3, 2, None, [1., 2., 1.])
        result = {'status': 'ok', 'characteristic': 2, 'coverage': None, 'edges': case.expected_edges(),
                  'intervals': [], 'simplices': case.expected_simplices()}
        self.assertIn([[0, 1, 2], 2.], result['simplices'])
        compare(case, result, result)
        with self.assertRaisesRegex(ValueError, 'simplex/value'):
            compare(case, result, {**result, 'simplices': result['simplices'][:-1]})
        wrong = [[vertices, 3. if len(vertices) == 3 else value] for vertices, value in result['simplices']]
        with self.assertRaisesRegex(ValueError, 'simplex/value'):
            compare(case, {**result, 'simplices': wrong}, result)

    def test_field_mismatch_and_reference_limits(self):
        case = Fixture('field', 'flag', 2, 0, None, [], characteristic=3)
        result = {'status': 'ok', 'characteristic': 3, 'coverage': None, 'edges': [], 'intervals': []}
        compare(case, result, result)
        with self.assertRaisesRegex(ValueError, 'field mismatch'):
            compare(case, result, {**result, 'characteristic': 2})
        self.assertIsNone(reference_exclusion('ripser', case))
        case.characteristic = 46337
        self.assertIsNone(reference_exclusion('gudhi', case))
        self.assertIsNotNone(reference_exclusion('ripser', case))
        case.characteristic = 4294967291
        self.assertIsNotNone(reference_exclusion('gudhi', case))

    def test_fixture_protocol_keeps_explicit_vertex_count(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'fixture.txt'
            Fixture('isolated', 'flag', 7, 1, None, []).write(path)
            self.assertEqual(path.read_text(), 'flag 7 1 none 0 2\n')
        cases = fixtures()
        self.assertEqual(len(cases), len({case.name for case in cases}))
        self.assertTrue(any(case.precision == 'float64' for case in cases))


if __name__ == '__main__':
    unittest.main()
