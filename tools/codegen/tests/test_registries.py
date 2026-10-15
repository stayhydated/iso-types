"""Registry boundaries and source-preserving selections."""

import hashlib
import json
from pathlib import Path
import tempfile
import unittest

from tools.codegen import iban, language_groups, language_relations, locations, numbering, registries, trade
from tools.codegen.output import ROOT
from tools.codegen.policy import CurrentData


class RegistryTests(unittest.TestCase):
    def test_every_pinned_source_is_verified_and_changed_bytes_fail(self):
        registries.verify_sources(ROOT / "data")
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "registries").mkdir()
            source = root / "registries/example.txt"
            source.write_bytes(b"original")
            (root / "registries/provenance.json").write_text(json.dumps({
                "as_of": "2026-10-15", "files": {"example.txt": {
                    "sha256": hashlib.sha256(b"original").hexdigest()}}}))
            registries.verify_sources(root)
            source.write_bytes(b"changed")
            with self.assertRaisesRegex(ValueError, "checksum mismatch"):
                registries.verify_sources(root)

    def test_iban_ranges_and_structures_reject_ambiguous_or_outside_positions(self):
        self.assertEqual(iban.components("4!a6!n8!c"), [(4, "a"), (6, "n"), (8, "c")])
        self.assertEqual(iban.identifier_range("1-4", 18), (0, 4))
        self.assertEqual(iban.identifier_range("5-18", 18), (4, 18))
        self.assertIsNone(iban.identifier_range("N/A", 18))
        for pattern in ("0!n", "4n", "4!x", "4!n?", ""):
            with self.subTest(pattern=pattern), self.assertRaises(ValueError):
                iban.components(pattern)
        for positions in ("0-4", "5-4", "1-19", "1,4"):
            with self.subTest(positions=positions), self.assertRaises(ValueError):
                iban.identifier_range(positions, 18)

    def test_numbering_inheritance_keeps_regional_and_non_decimal_choices(self):
        systems, preferences = numbering.records(ROOT / "data", CurrentData(ROOT / "data"))
        self.assertEqual(preferences["ar-MA"][:2], ("latn", "arab"))
        self.assertEqual(preferences["zh-Hant-TW"][2:], ("hant", "hantfin"))
        self.assertEqual(systems["latn"][1]["digits"], "0123456789")
        self.assertEqual(systems["hant"][1]["type"], "algorithmic")

    def test_group_hierarchies_and_macrolanguages_reject_cycles(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "registries").mkdir()
            (root / "registries/language-groups.html").write_text(
                "<table><tr><td>aaa</td><td>Group A</td><td></td><td></td>"
                "<td>bbb:aaa</td><td></td></tr></table>")
            with self.assertRaisesRegex(ValueError, "inconsistent"):
                language_groups.records(root)
            (root / "registries/language-subtag-registry.txt").write_text(
                "Type: language\nSubtag: aaa\nMacrolanguage: bbb\n%%\n"
                "Type: language\nSubtag: bbb\nMacrolanguage: aaa\n")
            with self.assertRaisesRegex(ValueError, "cyclic"):
                language_relations.macrolanguage_records(root)

    def test_location_alternatives_and_coordinate_precision_are_preserved(self):
        rows = locations.records(ROOT / "data", {"FI", "US"})
        self.assertGreater(len(rows["FIHEL"]), 1)
        self.assertGreater(len({row[6] for row in rows["USLEB"]}), 1)
        self.assertEqual(locations.coordinate_minutes("9000N 18000W"), (5400, -10800))
        self.assertEqual(locations.coordinate_minutes("6010N 02456E"), (3610, 1496))
        for text in ("9001N 18000E", "0060N 00000E", "0000X 00000E", "0000N 18100E", ""):
            self.assertIsNone(locations.coordinate_minutes(text), text)

    def test_trade_selection_keeps_package_counts_and_excludes_deleted_codes(self):
        units, packages = trade.build_catalogs(ROOT / "data")
        unit_codes = {entry.code for entry in units.entries}
        self.assertEqual(len(unit_codes), 2161)
        self.assertEqual(len(packages.entries), 405)
        self.assertTrue({"X" + entry.code for entry in packages.entries} <= unit_codes)
        self.assertNotIn("HAR", unit_codes)
        self.assertIn("KGM", unit_codes)


if __name__ == "__main__":
    unittest.main()
