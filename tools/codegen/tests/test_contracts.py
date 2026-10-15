"""Contracts for pinned selection metadata and deterministic regeneration."""

from pathlib import Path
import tempfile
import unittest

from tools.codegen.catalog import Catalog, CatalogSpec, Entry, read_ftl
from tools.codegen.output import ROOT, OutputPlan
from tools.codegen.policy import CurrentData, expand_range
from tools.codegen.rust import literal
from tools.codegen.relations import build_relations
from tools.codegen.units import build_units
from tools.codegen.contacts import address_rows, format_fields, phone_rows
from tools.codegen.preferences import regional_rows
from tools.codegen.supplemental import region_records, zone_records, build_supplemental
from tools.codegen.unit_preferences import groups, preference_rows
from tools.codegen.catalog import build_catalogs
import json


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
        self.assertEqual(policy.currency_regions["US"], ("USD",))
        self.assertEqual(policy.currency_regions["PA"], ("PAB", "USD"))
        self.assertEqual(policy.currency_regions["ZW"], ("USD", "ZWG"))
        self.assertEqual(policy.currency_regions["AQ"], ())

    def test_measurement_selection_requires_regular_codes_and_real_cldr_names(self):
        policy = CurrentData(ROOT / "data")
        units = build_units(ROOT / "data", policy)
        entries = {entry.code: entry for entry in units.entries}
        self.assertEqual(len(entries), 148)
        self.assertEqual(entries["meter"].name, "meters")
        self.assertEqual(entries["celsius"].metadata["category"], "temperature")
        self.assertNotIn("light-year", entries)
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "current").mkdir()
            (root / "current/units-en.xml").write_text("<ldml><units/></ldml>")
            (root / "current/unitPreferences.xml").write_bytes((ROOT / "data/current/unitPreferences.xml").read_bytes())
            with self.assertRaisesRegex(ValueError, "CLDR English unit name is unavailable"):
                build_units(root, policy)


class RelationshipTests(unittest.TestCase):
    @staticmethod
    def entry(code, **metadata):
        return Entry(code, code.replace("-", ""), code, code, (), metadata)

    def catalogs(self, subdivisions):
        return (
            Catalog(CatalogSpec("3166-1", "Countries", "code", "3166-1"),
                    (self.entry("CA"), self.entry("US"))),
            Catalog(CatalogSpec("4217", "Currencies", "code", "4217"),
                    (self.entry("CAD"), self.entry("USD"))),
            Catalog(CatalogSpec("3166-2", "Subdivisions", "code", "3166-2"), tuple(subdivisions)),
        )

    def test_parent_country_and_currency_relationships_are_typed_and_closed(self):
        parent = self.entry("US-AA", type="State")
        child = self.entry("US-BB", type="County", parent="US-AA")
        model = build_relations(self.catalogs([parent, child]), CurrentData(ROOT / "data"))
        self.assertEqual(model.country_roots["US"], (parent,))
        self.assertEqual(model.parents, (None, parent))
        self.assertEqual(model.children["US-AA"], (child,))
        self.assertEqual([entry.code for entry in model.country_currencies["US"]], ["USD"])
        self.assertEqual([entry.code for entry in model.currency_countries["CAD"]], ["CA"])

    def test_invalid_and_cyclic_subdivision_parents_fail_before_rendering(self):
        policy = CurrentData(ROOT / "data")
        cases = [
            [self.entry("ZZ-AA", type="State")],
            [self.entry("US-AA", type="State", parent="US-MISSING")],
            [self.entry("US-AA", type="State", parent="CA-BB"), self.entry("CA-BB", type="Province")],
            [self.entry("US-AA", type="State", parent="US-AA")],
            [self.entry("US-AA", type="State", parent="US-BB"), self.entry("US-BB", type="County", parent="US-AA")],
        ]
        for entries in cases:
            with self.subTest(codes=[entry.code for entry in entries]), self.assertRaises(ValueError):
                    build_relations(self.catalogs(entries), policy)


class SupplementalTests(unittest.TestCase):
    def test_sources_preserve_complete_relationships_and_country_defaults(self):
        data = ROOT / "data"
        policy = CurrentData(data)
        snapshot = json.loads((data / "icu/english.json").read_text())
        catalogs = build_catalogs(data, ROOT / "crates/iso-types/i18n/en/iso-types", policy, snapshot["catalogs"])
        countries = next(c for c in catalogs if c.spec.standard == "3166-1")
        preferences = dict(zip((e.code for e in countries.entries), regional_rows(data, countries)))
        self.assertEqual(preferences["US"]["firstDay"], "sun")
        self.assertEqual(preferences["GB"]["minDays"], "4")
        self.assertEqual(preferences["MM"]["temperature"], "metric")
        records = region_records(data)
        self.assertEqual(records["014"]["parent"], "202")
        self.assertEqual(records["202"]["parent"], "002")
        self.assertEqual(records["001"]["parent"], None)
        zones = zone_records(data)
        self.assertIn("US", zones["America/New_York"])
        self.assertEqual(zones["Etc/UTC"], ())
        for catalog in build_supplemental(data, snapshot):
            for field in ("code", "variant", "message_id"):
                self.assertEqual(len(catalog.entries), len({getattr(e, field) for e in catalog.entries}))

    def test_address_template_literals_and_new_landmark_fields_are_preserved(self):
        self.assertEqual(format_fields("%N%n%A%n%C %S %Z"), ["N", "A", "C", "S", "Z"])
        self.assertEqual(format_fields("%N%n%A%n%X%n%C%nGUERNSEY%n%Z"), ["N", "A", "X", "C", "Z"])
        with self.assertRaisesRegex(ValueError, "unknown address placeholder"):
            format_fields("%?")
        rows = address_rows(ROOT / "data")
        self.assertEqual(len(rows), 249)
        self.assertEqual(rows["MY"]["sublocality_name_type"], "village_township")
        self.assertTrue({"T", "F", "L"} <= set(rows["IN"]["fields"]))
        self.assertEqual(rows["SY"]["require"], "AC")

    def test_preference_rows_retain_order_mixed_units_and_world_defaults(self):
        data = ROOT / "data"
        height = next(g for g in groups(data) if g.get("usage") == "person-height")
        rows = preference_rows(height)
        self.assertEqual(rows["US"][0], (("foot", "inch"), 3.0, None))
        self.assertEqual(rows["001"][0][0], ("centimeter",))
        phone = phone_rows(data)
        regions = {r.get("id"): r.get("countryCode") for r in phone if r.get("id") != "001"}
        self.assertEqual(regions["US"], regions["CA"])
        self.assertEqual(regions["KZ"], regions["RU"])


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
