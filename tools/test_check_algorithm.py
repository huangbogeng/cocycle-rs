"""Focused checks must fail visibly instead of reporting unverified success."""

import contextlib
import io
from pathlib import Path
import subprocess
import unittest
from unittest.mock import patch

import check_algorithm


class AlgorithmChecksTests(unittest.TestCase):
    def test_failed_step_stops_before_later_checks(self):
        failure = subprocess.CalledProcessError(7, ["check_source"])
        with patch.object(check_algorithm, "run", side_effect=failure) as run:
            with contextlib.redirect_stderr(io.StringIO()) as errors:
                self.assertEqual(check_algorithm.main(["diagram-analysis"]), 1)
        self.assertEqual(run.call_count, 1)
        self.assertIn("Algorithm checks failed", errors.getvalue())

    def test_missing_tool_is_a_reported_failure(self):
        with patch.object(check_algorithm, "run", side_effect=FileNotFoundError("cargo")):
            with contextlib.redirect_stderr(io.StringIO()) as errors:
                self.assertEqual(check_algorithm.main(["diagram-analysis"]), 1)
        self.assertIn("cargo", errors.getvalue())

    def test_artifact_uses_cargos_path_and_requires_library_output(self):
        output = ('{"reason":"compiler-artifact","target":{"name":"cocycle"},'
                  '"filenames":["custom target/debug/libcocycle.rlib"]}\n')
        with patch.object(check_algorithm, "run", return_value=subprocess.CompletedProcess(
                [], 0, stdout=output)):
            self.assertEqual(check_algorithm.library_artifact(),
                             Path("custom target/debug/libcocycle.rlib"))
        with patch.object(check_algorithm, "run", return_value=subprocess.CompletedProcess(
                [], 0, stdout='{"reason":"build-finished","success":true}\n')):
            with self.assertRaisesRegex(ValueError, "library artifact"):
                check_algorithm.library_artifact()


if __name__ == "__main__":
    unittest.main()
