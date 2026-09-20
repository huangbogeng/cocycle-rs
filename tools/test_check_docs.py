"""Regression checks for nested documentation discovery and relative links."""

from contextlib import redirect_stderr, redirect_stdout
from io import StringIO
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import check_docs


class DocumentationTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name).resolve()
        root_patch = patch.object(check_docs, "ROOT", self.root)
        root_patch.start()
        self.addCleanup(root_patch.stop)

    def write(self, name, text):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")

    def run_check(self):
        stdout, stderr = StringIO(), StringIO()
        with redirect_stdout(stdout), redirect_stderr(stderr):
            status = check_docs.main()
        return status, stdout.getvalue(), stderr.getvalue()

    def test_nested_pages_resolve_parent_links_and_heading_fragments(self):
        self.write("docs/README.md", "[Guide](guides/rips.md)\n")
        self.write("docs/guides/rips.md",
                   "[Math](../reference/rips/mathematics.md#coverage)\n")
        self.write("docs/reference/rips/mathematics.md",
                   "# Coverage\n[Documentation](../../README.md)\n"
                   "[Again](#coverage)\n")
        status, stdout, stderr = self.run_check()
        self.assertEqual(status, 0, stderr)
        self.assertIn("3 Markdown files", stdout)

    def test_nested_errors_are_not_silently_skipped(self):
        self.write("docs/README.md", "# Documentation\n")
        self.write("docs/reference/rips/math.md",
                   "# Math\n[Missing](gone.md)\n"
                   "[Heading](../../README.md#missing)\n\u4e2d\u6587\n")
        status, _, stderr = self.run_check()
        self.assertEqual(status, 1)
        self.assertIn("missing local target gone.md", stderr)
        self.assertIn("missing heading ../../README.md#missing", stderr)
        self.assertIn("math.md:4: project documentation must be English", stderr)

    def test_code_examples_and_generated_artifacts_are_not_link_sources(self):
        self.write("docs/guides/rips.md",
                   "# Guide\n```text\n[Placeholder](missing.md)\n```\n"
                   "[External](https://example.invalid/not-fetched)\n")
        self.write("benches/results/run/notes.md", "[Old](missing.md)\n")
        self.write("target/generated.md", "[Generated](missing.md)\n")
        status, stdout, stderr = self.run_check()
        self.assertEqual(status, 0, stderr)
        self.assertIn("1 Markdown files", stdout)

    def test_benchmark_reports_and_native_guides_are_checked(self):
        self.write("benches/native/README.md", "[Missing](gone.cpp)\n")
        self.write("benches/reports/nested/report.md", "[Missing](gone.json)\n")
        status, _, stderr = self.run_check()
        self.assertEqual(status, 1)
        self.assertIn("benches/native/README.md", stderr)
        self.assertIn("benches/reports/nested/report.md", stderr)

    def test_reference_links_resolve_local_targets_and_headings(self):
        self.write("docs/reference/math.md", "# Coverage\n")
        self.write("docs/README.md",
                   '[Full][Math] [math][] [math]\n'
                   '[math]: reference/math.md#coverage "Contract"\n'
                   '[Remote][external]\n'
                   '[external]: https://example.invalid/not-fetched\n')
        status, _, stderr = self.run_check()
        self.assertEqual(status, 0, stderr)

    def test_reference_errors_include_original_line_numbers(self):
        self.write("docs/README.md",
                   '# Docs\n```text\n[Fake][missing]\n```\n'
                   '[Real][absent]\n[collapsed][]\n'
                   '[math]: missing.md\n[MATH]: #absent\n')
        status, _, stderr = self.run_check()
        self.assertEqual(status, 1)
        self.assertIn("README.md:5: undefined reference [absent]", stderr)
        self.assertIn("README.md:6: undefined reference [collapsed]", stderr)
        self.assertIn("README.md:7: missing local target missing.md", stderr)
        self.assertIn("README.md:8: duplicate reference definition [MATH]", stderr)
        self.assertIn("README.md:8: missing heading #absent", stderr)

    def test_code_spans_and_indented_longer_fences_are_not_links(self):
        self.write("docs/README.md",
                   '# Docs\n`[Inline](missing.md)`\n'
                   '``[Inline ` tick][missing]``\n'
                   '  ~~~text\n[Fake][missing]\n  ~~~~\n'
                   '[Real][absent]\n')
        status, _, stderr = self.run_check()
        self.assertEqual(status, 1)
        self.assertEqual(stderr.count("undefined reference"), 1)
        self.assertIn("README.md:7: undefined reference [absent]", stderr)
        self.assertNotIn("missing.md", stderr)

    def test_angle_destinations_and_normalized_reference_labels(self):
        self.write("docs/math notes.md", "# Coverage\n")
        self.write("docs/README.md",
                   '[Inline](<math notes.md#coverage>)\n'
                   '[Reference][MaTh  Notes]\n'
                   '[math notes]: <math notes.md#coverage> "Title"\n')
        status, _, stderr = self.run_check()
        self.assertEqual(status, 0, stderr)


if __name__ == "__main__":
    unittest.main()
