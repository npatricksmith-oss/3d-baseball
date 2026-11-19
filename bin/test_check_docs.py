import importlib.util
import tempfile
import unittest
from pathlib import Path

spec = importlib.util.spec_from_file_location("check_docs", Path(__file__).with_name("check-docs.py"))
cd = importlib.util.module_from_spec(spec)
spec.loader.exec_module(cd)


def run(files, include_dated=False):
    with tempfile.TemporaryDirectory() as d:
        root = Path(d)
        for name, body in files.items():
            p = root / name
            p.parent.mkdir(parents=True, exist_ok=True)
            p.write_text(body)
        return cd.check(root, include_dated)


class Slug(unittest.TestCase):
    def test_real_headings(self):
        self.assertEqual(
            cd.slugify("7. Older open items (from 2026-10-31 to 11-12; spot-checked against the code 2026-11-17)"),
            "7-older-open-items-from-2026-10-31-to-11-12-spot-checked-against-the-code-2026-11-17",
        )
        self.assertEqual(cd.slugify("Under_score & `code` Heading"), "under_score--code-heading")


class Check(unittest.TestCase):
    def test_clean(self):
        self.assertEqual(run({"A.md": "# Top\n[ok](B.md#sec) [self](#top)\n", "B.md": "## Sec\n"}), [])

    def test_dead_link(self):
        p = run({"A.md": "x\n[gone](docs/nope.md)\n"})
        self.assertEqual(len(p), 1)
        self.assertTrue(p[0].startswith("A.md:2:"))

    def test_dead_anchor(self):
        p = run({"A.md": "[x](B.md#missing)\n", "B.md": "# Real\n"})
        self.assertEqual(len(p), 1)
        self.assertIn("dead anchor", p[0])

    def test_duplicate_heading_anchor(self):
        files = {"B.md": "# Notes\n# Notes\n"}
        self.assertEqual(run({"A.md": "[x](B.md#notes-1)\n", **files}), [])
        self.assertEqual(len(run({"A.md": "[x](B.md#notes-2)\n", **files})), 1)

    def test_line_anchor_on_source(self):
        files = {"A.md": "[x](sim/a.rs#L12) `sim/a.rs:12` `sim/a.rs#L3-L9`\n", "sim/a.rs": "fn main(){}\n"}
        self.assertEqual(run(files), [])
        self.assertEqual(len(run({"A.md": "[x](sim/b.rs#L12)\n"})), 1)

    def test_fenced_block_ignored(self):
        body = "```\n[dead](nope.md) `sim/none.rs`\n```\n~~~\n[dead](nope2.md)\n~~~\n"
        self.assertEqual(run({"A.md": body}), [])

    def test_backticked_path(self):
        self.assertEqual(len(run({"A.md": "`docs/gone.md` `sim/*.rs` `docs/<x>.md` `foo/bar`\n"})), 1)

    def test_external_skipped(self):
        self.assertEqual(run({"A.md": "[x](https://example.com/a#b) [m](mailto:a@b.c)\n"}), [])

    def test_tasks_cap(self):
        self.assertEqual(run({"TASKS.md": "x" * cd.TASKS_MAX_BYTES}), [])
        p = run({"TASKS.md": "x" * (cd.TASKS_MAX_BYTES + 1)})
        self.assertEqual(len(p), 1)
        self.assertIn("exceeds", p[0])

    def test_dated_only_with_all(self):
        files = {"docs/reviews/r.md": "[x](gone.md)\n"}
        self.assertEqual(run(files), [])
        self.assertEqual(len(run(files, include_dated=True)), 1)


if __name__ == "__main__":
    unittest.main()
