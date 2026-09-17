"""Scaling experiment validation and resource boundaries; standard library only."""
import copy
import csv
import json
import math
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

import benchmark_scaling as scaling


def completed(diagram):
    return dict(copy.deepcopy(diagram), status="completed", samples_ms=[1., 2., 3.],
                hwm_before_kib=100, peak_rss_kib=120, rss_before_kib=100)


class ScalingTests(unittest.TestCase):
    def test_families_are_reproducible_finite_and_have_valid_shapes(self):
        for family in scaling.FAMILIES:
            case = scaling.make_case(family, 16)
            self.assertEqual(case, scaling.make_case(family, 16))
            self.assertEqual(len(case.values), 120)
            self.assertTrue(all(math.isfinite(x) and x >= 0 for x in case.values))
            self.assertEqual(case, scaling.bench.float32_case(case))
        with self.assertRaises(ValueError):
            scaling.make_case("grid", 15)

    def test_bipartite_graph_has_no_triangles_and_correct_cycle_rank(self):
        for n in [2, 5, 8]:
            case = scaling.make_case("bipartite", n)
            matrix = [[0.] * n for _ in range(n)]
            offset = 0
            for i in range(n):
                for j in range(i):
                    matrix[i][j] = matrix[j][i] = case.values[offset]
                    offset += 1
            edges = sum(x == 1 for x in case.values)
            self.assertFalse(any(max(matrix[a][b], matrix[a][c], matrix[b][c]) <= 1
                                 for a in range(n) for b in range(a) for c in range(b)))
            for cutoff in [None, 1.]:
                expected = scaling.bipartite_diagram(n, cutoff)
                bars = [x for x in expected["intervals"] if x[0] == 1]
                self.assertEqual(len(bars), edges - n + 1)
                self.assertTrue(all(x == [1, 1., "F", 2.] if cutoff is None else x == [1, 1., "C", 1.] for x in bars))
            if n == 2:
                self.assertEqual(scaling.bipartite_diagram(n, 1.)["coverage"], ["complete", None])

    def test_external_results_are_compared_when_cocycle_times_out(self):
        expected = scaling.bipartite_diagram(8, None)
        record = {"results": {"cocycle": {"status": "timeout"},
                              "ripser_py": completed(expected), "gudhi_simplex_tree": completed(expected)}}
        scaling.validate_record(record)
        self.assertEqual(record["validation"], "passed")
        record["results"]["ripser_py"]["intervals"].pop()
        with self.assertRaises(AssertionError):
            scaling.validate_record(record)

    def test_analytic_oracle_can_validate_a_single_survivor(self):
        expected = scaling.bipartite_diagram(8, 1.)
        record = {"results": {"cocycle": completed(expected)}}
        scaling.validate_record(record)
        self.assertEqual(record["validation"], "unverified")
        scaling.validate_record(record, expected)
        self.assertEqual(record["validation"], "passed")
        record["results"]["cocycle"]["intervals"][-1][2] = "E"
        with self.assertRaises(AssertionError):
            scaling.validate_record(record, expected)

    def test_unverified_and_failed_runs_cannot_generate_speedup_claims(self):
        expected = scaling.bipartite_diagram(8, None)
        record = {"case": "test", "n": 8, "results": {"cocycle": {"status": "timeout"},
                  "ripser_py": completed(expected)}, "validation": "unverified"}
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "summary.csv"
            scaling.summarize(path, [record])
            with path.open() as stream:
                rows = list(csv.DictReader(stream))
            self.assertTrue(all(not r["median_ms"] and not r["cocycle_over_backend"] for r in rows))
            # Partial worker execution must still be serializable after failure.
            del record["results"]["cocycle"]
            scaling.summarize(path, [record])
            record["validation"] = "mismatch"
            scaling.summarize(path, [record])
            with path.open() as stream:
                self.assertFalse(list(csv.DictReader(stream))[0]["median_ms"])

    @unittest.skipUnless(sys.platform == "linux", "RLIMIT_AS experiment is Linux-only")
    def test_limit_wrapper_applies_address_space_limit_to_execed_worker(self):
        output = subprocess.check_output([
            sys.executable, str(Path(scaling.__file__).resolve()), "--limited-worker", "128",
            sys.executable, "-c", "import json, resource; print(json.dumps(resource.getrlimit(resource.RLIMIT_AS)))",
        ], text=True, timeout=10)
        self.assertEqual(json.loads(output), [128 * 1024**2] * 2)


if __name__ == "__main__":
    unittest.main()
