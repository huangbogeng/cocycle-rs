"""Distance transport, independent expectations and failure-retention regressions."""

import math
from pathlib import Path
import struct
import sys
import tempfile
import unittest

from distance_common import (MAGIC, agrees, invoke, read_fixture, tiny_oracle,
                             tolerance, unpack_number, write_fixture)


class DistanceProtocolTests(unittest.TestCase):
    def test_round_trip_preserves_multiplicity_and_essential_points(self):
        first = [(-0.0, 1.0), (0.0, 1.0), (2.0, math.inf)]
        second = []
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'fixture.bin'
            write_fixture(path, first, second)
            actual, other = read_fixture(path)
            self.assertEqual((actual, other), (first, second))
            self.assertEqual(math.copysign(1.0, actual[0][0]), -1.0)
            path.write_bytes(path.read_bytes() + b'\0')
            with self.assertRaisesRegex(ValueError, 'length'):
                read_fixture(path)

    def test_huge_claimed_length_is_rejected_before_allocation(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'fixture.bin'
            path.write_bytes(MAGIC + struct.pack('<QQ', 2 ** 64 - 1, 0))
            with self.assertRaises(ValueError):
                read_fixture(path)

    def test_hand_derived_diagonal_and_pair_costs(self):
        for metric, diagonal, pair in [('bottleneck', 1.0, 1.0), ('w1', 1.0, 1.0),
                                       ('w2', math.sqrt(2), math.sqrt(2))]:
            self.assertEqual(tiny_oracle([(0.0, 2.0)], [], metric), diagonal)
            self.assertEqual(tiny_oracle([(0.0, 2.0)], [(1.0, 3.0)], metric), pair)

    def test_partial_matching_and_duplicate_multiplicity(self):
        for metric, expected in [('bottleneck', 1.0), ('w1', 2.0), ('w2', 2.0)]:
            self.assertEqual(tiny_oracle([(0.0, 2.0)] * 3, [(0.0, 2.0)], metric), expected)
        self.assertEqual(tiny_oracle([(0.0, 0.0)], [], 'w1'), 0.0)

    def test_essential_counts_and_order(self):
        self.assertEqual(tiny_oracle([(0.0, math.inf)], [], 'w1'), math.inf)
        first = [(3.0, math.inf), (0.0, math.inf), (0.0, 2.0)]
        second = [(1.0, math.inf), (5.0, math.inf)]
        self.assertEqual(tiny_oracle(first, second, 'w1'), 4.0)
        self.assertEqual(tiny_oracle(first, second, 'w2'), math.sqrt(7.0))

    def test_adjacent_float_stress_cannot_be_hidden_by_tolerance(self):
        case = {'first': [(1.0, math.nextafter(1.0, math.inf))], 'second': [], 'exact_grid': True}
        for metric in ('bottleneck', 'w1', 'w2'):
            expected = tiny_oracle(case['first'], case['second'], metric)
            self.assertFalse(agrees(expected * 2, expected, case, metric))
            self.assertLess(tolerance(case, metric, expected), expected)

    def test_invalid_input_and_scalar_are_rejected(self):
        for points in ([(math.nan, 1.0)], [(2.0, 1.0)], [(-math.inf, 1.0)]):
            with self.assertRaises(ValueError):
                tiny_oracle(points, [], 'w1')
        for value in ({'value_kind': 'finite', 'value': math.nan},
                      {'value_kind': 'finite', 'value': True},
                      {'value_kind': 'infinite', 'value': 1}):
            with self.assertRaises(ValueError):
                unpack_number(value)

    def test_process_error_protocol_error_and_timeout_remain_distinct(self):
        cases = [(['-c', 'import sys; print("failure"); sys.exit(3)'], 'process_error', 5),
                 (['-c', 'print("not JSON")'], 'protocol_error', 5),
                 (['-c', 'import time; time.sleep(10)'], 'timeout', 0.02)]
        for arguments, expected, timeout in cases:
            sample = invoke([sys.executable, *arguments], 'unused.bin', 'w1', timeout=timeout)
            self.assertEqual(sample['status'], expected)


if __name__ == '__main__':
    unittest.main()
