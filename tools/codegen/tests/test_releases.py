import unittest

from tools.codegen.releases import (
    BY_NAME, cldr_tag, libphonenumber_tag, phonenumber_metadata, tz_tag, verify_agreement, verify_pins,
)


def pins(revision: str) -> dict:
    return {
        "cldr": {"version": "49.0.0", "tag": "release-49", "revision": revision},
        "tz": {"version": "2026e", "tag": "2026e", "revision": revision},
        "libphonenumber": {"version": "9.0.33", "tag": "v9.0.33", "revision": revision},
    }


class ReleasePinTest(unittest.TestCase):
    def test_versions_map_to_release_tags(self):
        self.assertEqual(cldr_tag("49.0.0"), "release-49")
        self.assertEqual(cldr_tag("48.2.0"), "release-48-2")
        self.assertEqual(cldr_tag("47.0.1"), "release-47-0-1")
        self.assertEqual(tz_tag("2026e"), "2026e")
        self.assertEqual(libphonenumber_tag("9.0.33"), "v9.0.33")
        for tag, version in ((cldr_tag, "49"), (tz_tag, "2026"), (libphonenumber_tag, "9.0")):
            with self.assertRaises(ValueError):
                tag(version)

    def test_every_released_input_uses_its_pinned_commit(self):
        revision = "a" * 40
        provenance = {
            "releases": pins(revision),
            "files": {
                "supplementalData.xml": {"url": BY_NAME["cldr"].raw + revision + "/common/supplementalData.xml"},
                "zone.tab": {"url": BY_NAME["tz"].raw + revision + "/zone.tab"},
                "addresses.json": {"url": "https://www.gstatic.com/addresses/{country}"},
            },
        }
        self.assertEqual(verify_pins(provenance)["tz"]["tag"], "2026e")
        provenance["files"]["backward"] = {"url": BY_NAME["tz"].raw + "b" * 40 + "/backward"}
        with self.assertRaisesRegex(ValueError, "not pinned to tz"):
            verify_pins(provenance)
        del provenance["files"]["backward"]
        provenance["releases"]["cldr"]["tag"] = "release-48"
        with self.assertRaisesRegex(ValueError, "does not name"):
            verify_pins(provenance)
        del provenance["releases"]["cldr"]
        with self.assertRaisesRegex(ValueError, "must cover"):
            verify_pins(provenance)

    def test_pins_must_equal_the_releases_bundled_by_locked_crates(self):
        packages = [
            {"name": "icu_time_data", "version": "2.4.0", "metadata": {"sources": {"cldr": {"tagged": "49.0.0"}}}},
            {"name": "phonenumber", "version": "0.3.10+9.0.33", "metadata": None},
        ]
        self.assertEqual(phonenumber_metadata(packages), {"9.0.33"})
        with self.assertRaisesRegex(ValueError, r"tz 2026e differs from jiff-tzdb \(none\)"):
            verify_agreement(pins("a" * 40), packages)


if __name__ == "__main__":
    unittest.main()
