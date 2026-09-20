"""Regression checks for source hygiene and immutable evidence exclusions."""

from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import check_source


class SourceTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name).resolve()
        root_patch = patch.object(check_source, "ROOT", self.root)
        root_patch.start()
        self.addCleanup(root_patch.stop)

    def write(self, name, data):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)
        return path

    def test_checks_only_maintained_source_and_never_rewrites(self):
        code = self.write("src/geometry/mod.rs", b"//! Geometry.\n")
        tool = self.write("tools/check.py", b"raise RuntimeError('must not import')\n")
        config = self.write(".editorconfig", b"root = true\n")
        raw = self.write("benches/results/run/source.py", b"not valid code\r\n")
        downloaded = self.write("target/native-sources/upstream/source.cpp", b"\xff")
        cache = self.write("tools/__pycache__/check.pyc", b"\xff")
        paths = check_source.sources()
        self.assertEqual(set(paths), {code, tool, config})
        for path in paths:
            self.assertEqual(check_source.check(path), [])
        self.assertEqual(raw.read_bytes(), b"not valid code\r\n")
        self.assertEqual(downloaded.read_bytes(), cache.read_bytes())

    def test_reports_encoding_whitespace_and_syntax(self):
        invalid = self.write("src/invalid.rs", b"\xff")
        self.assertIn("expected UTF-8", check_source.check(invalid)[0])
        path = self.write("tools/broken.py", b"\xef\xbb\xbfif True\r\n\tpass \r\n# end")
        errors = "\n".join(check_source.check(path))
        for expected in ("BOM", "LF line endings", "final newline", "trailing whitespace",
                         "spaces for indentation", "Python syntax"):
            self.assertIn(expected, errors)

    def test_valid_python_is_parsed_without_execution(self):
        path = self.write("tools/valid.py", b"def fail():\n    return 1 / 0\nfail()\n")
        self.assertEqual(check_source.check(path), [])
        path.write_bytes(b"def broken(\n")
        self.assertIn("Python syntax", "\n".join(check_source.check(path)))


if __name__ == "__main__":
    unittest.main()
