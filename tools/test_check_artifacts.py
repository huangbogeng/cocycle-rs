"""Exercise artifact admission against real temporary Git indexes."""

from pathlib import Path
import subprocess
import tempfile
import unittest

import check_artifacts


class ArtifactTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.git("init", "-q")

    def git(self, *args):
        return subprocess.check_output(["git", *args], cwd=self.root, stderr=subprocess.PIPE)

    def stage(self, name, data):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)
        self.git("add", "-f", "--", name)

    def check(self):
        return check_artifacts.check(self.root)

    def test_sources_reports_and_small_test_fixtures_are_allowed(self):
        for name in ("src/lib.rs", "benches/reports/commit-example.md", "tests/fixtures/square.json"):
            self.stage(name, b"fixture\n")
        self.assertEqual(self.check(), [])

    def test_forced_generated_files_and_relocated_archives_are_rejected(self):
        names = ("benches/results/new/summary.json", "target/sample.txt", "tools/__pycache__/x.pyc",
                 "benches/reports/evidence.tar.gz", "build.log", "worker.exe")
        for name in names:
            self.stage(name, b"output\n")
        errors = self.check()
        self.assertEqual(len(errors), len(names))
        for name in names:
            self.assertTrue(any(error.startswith(name + ":") for error in errors))

    def test_old_artifacts_have_no_exemption_and_staged_removal_passes(self):
        name = "benches/results/old-run/results.json"
        self.stage(name, b"old output\n")
        self.assertEqual(len(self.check()), 1)
        self.git("rm", "-f", "--", name)
        self.assertEqual(self.check(), [])

    def test_untracked_local_output_is_not_mistaken_for_a_submission(self):
        path = self.root / "target" / "run.log"
        path.parent.mkdir()
        path.write_text("local diagnostic\n")
        self.assertEqual(self.check(), [])


if __name__ == "__main__":
    unittest.main()
