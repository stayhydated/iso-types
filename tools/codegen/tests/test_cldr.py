import unittest

from tools.codegen.cldr import RAW, tag_for, verify_pin


class CldrPinTest(unittest.TestCase):
    def test_versions_map_to_release_tags(self):
        self.assertEqual(tag_for("49.0.0"), "release-49")
        self.assertEqual(tag_for("48.2.0"), "release-48-2")
        self.assertEqual(tag_for("47.0.1"), "release-47-0-1")
        with self.assertRaises(ValueError):
            tag_for("49")

    def test_every_cldr_input_uses_the_pinned_commit(self):
        revision = "a" * 40
        provenance = {
            "cldr": {"version": "49.0.0", "tag": "release-49", "revision": revision},
            "files": {
                "supplementalData.xml": {"url": RAW + revision + "/common/supplemental/supplementalData.xml"},
                "zone.tab": {"url": "https://raw.githubusercontent.com/eggert/tz/b/zone.tab"},
            },
        }
        self.assertEqual(verify_pin(provenance)["tag"], "release-49")
        provenance["files"]["likelySubtags.xml"] = {"url": RAW + "b" * 40 + "/common/x.xml"}
        with self.assertRaisesRegex(ValueError, "not pinned"):
            verify_pin(provenance)
        provenance["cldr"]["tag"] = "release-48"
        with self.assertRaisesRegex(ValueError, "does not name"):
            verify_pin(provenance)


if __name__ == "__main__":
    unittest.main()
