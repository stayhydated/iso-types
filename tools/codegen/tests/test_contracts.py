"""Contracts for pinned selection metadata and deterministic regeneration."""

from pathlib import Path
import tempfile
import unittest

from tools.codegen.catalog import read_ftl
from tools.codegen.output import ROOT, OutputPlan
from tools.codegen.policy import CurrentData, expand_range
from tools.codegen.rust import literal


class RangeTests(unittest.TestCase):
    def test_abbreviated_and_carried_ranges(self):
        self.assertEqual(expand_range("001~3"), {"001", "002", "003"})
        self.assertEqual(expand_range("Hans~t"), {"Hans", "Hant"})
        self.assertEqual(expand_range("az~bb"), {"az", "ba", "bb"})
        self.assertEqual(expand_range("en"), {"en"})

    def test_malformed_ranges_fail(self):
        for token in ("abc~", "~abc", "aa~aaa", "zz~aa", "aa~A", "a!~z"):
            with self.subTest(token=token), self.assertRaises(ValueError):
                expand_range(token)


class SelectionTests(unittest.TestCase):
    def test_current_representations_and_excluded_components(self):
        policy = CurrentData(ROOT / "data")
        self.assertEqual(policy.language("eng"), "en")
        self.assertEqual(policy.language("fra"), "fr")
        self.assertIsNone(policy.language("fre"))
        self.assertIsNone(policy.language("iw"))
        self.assertTrue({"Latn", "Hans", "Hant"} <= policy.scripts)
        self.assertTrue({"Egyp", "Qaai", "Zinh"}.isdisjoint(policy.scripts))
        self.assertTrue({"EUR", "USD", "ZWG"} <= policy.currencies)
        self.assertTrue({"BGN", "XTS", "XXX", "XAU"}.isdisjoint(policy.currencies))


class OutputTests(unittest.TestCase):
    def test_check_reports_every_difference_without_writing(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            existing, missing = root / "existing.rs", root / "nested/missing.rs"
            existing.write_text("old\n", encoding="utf-8")
            plan = OutputPlan()
            plan.add(existing, "new\n")
            plan.add(missing, "generated\n")
            self.assertEqual(plan.apply(check=True), [existing, missing])
            self.assertEqual(existing.read_text(encoding="utf-8"), "old\n")
            self.assertFalse(missing.parent.exists())
            self.assertEqual(plan.apply(check=False), [existing, missing])
            self.assertEqual(existing.read_text(encoding="utf-8"), "new\n")
            self.assertEqual(missing.read_text(encoding="utf-8"), "generated\n")
            self.assertEqual(plan.apply(check=True), [])

    def test_duplicate_outputs_are_rejected(self):
        plan = OutputPlan()
        plan.add(Path("catalog.rs"), "first")
        with self.assertRaises(ValueError):
            plan.add(Path("catalog.rs"), "second")

    def test_imported_snapshots_preserve_the_bytes_used_for_checksums(self):
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "source.json"
            source = b'{\r\n  "name": "Source"\r\n}\r\n'
            plan = OutputPlan()
            plan.add(path, source)
            plan.apply(check=False)
            self.assertEqual(path.read_bytes(), source)
            self.assertEqual(plan.apply(check=True), [])

    def test_rust_literals_preserve_unicode_and_escape_rust_syntax(self):
        self.assertEqual(literal('quote " slash \\ newline\n tab\t'), '"quote \\" slash \\\\ newline\\n tab\\t"')
        self.assertEqual(literal("日本語\x01\x7f"), '"日本語\\u{1}\\u{7f}"')
        with self.assertRaises(ValueError):
            literal("\ud800")

    def test_owned_ftl_requires_unique_literal_messages(self):
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "catalog.ftl"
            path.write_text("## Names\n\nkey = Authoritative name\n", encoding="utf-8")
            self.assertEqual(read_ftl(path), {"key": "Authoritative name"})
            for source in ("key = One\nkey = Two\n", "key = { $name }\n", "invalid\n"):
                path.write_text(source, encoding="utf-8")
                with self.subTest(source=source), self.assertRaises(ValueError):
                    read_ftl(path)


if __name__ == "__main__":
    unittest.main()
