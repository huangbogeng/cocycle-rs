"""Guard the benchmark's validation gate; no external packages required."""
import copy
import unittest

from benchmark_gudhi import Case, compare, float32_case


class DiagramComparisonTests(unittest.TestCase):
    def setUp(self):
        self.diagram = {"coverage": ["through", 1.],
                        "intervals": [[0, 0., "F", 1.], [0, 0., "F", 1.], [1, 1., "C", 1.]]}

    def test_order_does_not_change_multiset(self):
        other = copy.deepcopy(self.diagram)
        other["intervals"].reverse()
        compare(self.diagram, other, "distances")

    def test_lost_multiplicity_fails(self):
        other = copy.deepcopy(self.diagram)
        other["intervals"].pop(0)
        with self.assertRaisesRegex(AssertionError, "count"):
            compare(self.diagram, other, "distances")

    def test_essential_censored_and_dimension_mismatch_fail(self):
        for index, value in [(0, 0), (2, "E")]:
            with self.subTest(index=index):
                other = copy.deepcopy(self.diagram)
                other["intervals"][-1][index] = value
                with self.assertRaises(AssertionError):
                    compare(self.diagram, other, "distances")

    def test_coverage_mismatch_fails(self):
        other = copy.deepcopy(self.diagram)
        other["coverage"] = ["complete", None]
        with self.assertRaisesRegex(AssertionError, "coverage"):
            compare(self.diagram, other, "distances")

    def test_matrix_comparison_is_exact_but_point_norms_allow_roundoff(self):
        other = copy.deepcopy(self.diagram)
        other["intervals"][0][3] += 1e-15
        with self.assertRaisesRegex(AssertionError, "endpoint"):
            compare(self.diagram, other, "distances")
        compare(self.diagram, other, "points")
        other["intervals"][0][3] += 1e-6
        with self.assertRaisesRegex(AssertionError, "endpoint"):
            compare(self.diagram, other, "points")


class PrecisionPolicyTests(unittest.TestCase):
    def test_distance_and_cutoff_are_quantized_together_without_mutating_original(self):
        original = Case("near_cutoff", 3, [1., 1. + 2**-25, 1. + 2**-22], cutoff=1. + 2**-26)
        rounded = float32_case(original)
        self.assertEqual(rounded.values[:2], [1., 1.])
        self.assertEqual(rounded.cutoff, 1.)
        self.assertGreater(rounded.values[2], rounded.cutoff)
        self.assertGreater(original.values[1], original.cutoff)

    def test_quantization_is_idempotent_and_keeps_complete_range(self):
        rounded = float32_case(Case("pair", 2, [.2]))
        self.assertNotEqual(rounded.values, [.2])
        self.assertEqual(float32_case(rounded), rounded)
        self.assertIsNone(rounded.cutoff)

    def test_point_clouds_cannot_claim_exact_float32_filtration_parity(self):
        with self.assertRaises(ValueError):
            float32_case(Case("point", 1, [0., 0.], mode="points"))

    def test_unrepresentable_quantization_fails(self):
        for value in [float("nan"), float("inf"), 1e300]:
            with self.subTest(value=value), self.assertRaises((ValueError, OverflowError)):
                float32_case(Case("pair", 2, [value]))


if __name__ == "__main__":
    unittest.main()
