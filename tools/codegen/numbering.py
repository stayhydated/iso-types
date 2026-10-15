"""Resolve CLDR numbering systems and locale inheritance from pinned XML."""

from pathlib import Path
import xml.etree.ElementTree as ET
from zipfile import ZipFile

from .catalog import Catalog, CatalogSpec, Entry
from .output import GENERATED
from .registries import validate
from .relations import variant_name
from .render import string_table, value_table, metadata_enum
from .rust import literal


def records(data: Path, policy):
    with ZipFile(data / "registries/numbering-locales.zip") as archive:
        locales = {Path(name).stem: ET.fromstring(archive.read(name)) for name in archive.namelist()}
    supplemental = ET.parse(data / "current/supplementalData.xml")
    parents = {child: row.get("parent") for group in supplemental.findall(".//parentLocales")
               if not group.get("component") for row in group.findall("parentLocale")
               for child in row.get("locales").split()}
    resolved = {}

    def resolve(locale, visiting=()):
        if locale in resolved:
            return resolved[locale]
        if locale in visiting:
            raise ValueError(f"cyclic numbering locale inheritance: {locale}")
        parent = parents.get(locale, locale.rsplit("_", 1)[0] if "_" in locale else "root")
        values = {} if locale == "root" else dict(resolve(parent, (*visiting, locale)))
        source = locales.get(locale)
        if source is not None:
            alias = source.find("alias")
            if alias is not None:
                values = dict(resolve(alias.get("source"), (*visiting, locale)))
            for row in source.findall("./numbers/defaultNumberingSystem"):
                if not row.get("alt") and row.text != "↑↑↑":
                    values["default"] = row.text
            for row in source.findall("./numbers/otherNumberingSystems/*"):
                if not row.get("alt") and row.text != "↑↑↑":
                    values[row.tag] = row.text
        if "default" not in values:
            raise ValueError(f"missing default numbering system: {locale}")
        resolved[locale] = values
        return values

    preferences = {}
    for locale in sorted(locales):
        if locale == "root":
            continue
        parts = locale.split("_")
        if parts[0] not in policy.languages:
            continue
        # Locale APIs share the catalog's current country/script selection.
        if any((len(p) == 4 and p not in policy.scripts) or
               (len(p) in {2, 3} and p not in policy.regions) for p in parts[1:]):
            continue
        values = resolve(locale)
        native = values.get("native", values["default"])
        preferences[locale.replace("_", "-")] = (values["default"], native,
            values.get("traditional", native), values.get("finance", values["default"]))
    selected = {code for row in preferences.values() for code in row}
    names = {row.get("type"): row.text for row in locales["en"].findall('.//type[@key="numbers"]')
             if not row.get("alt") and not row.get("scope")}
    systems = {}
    for row in ET.parse(data / "registries/numberingSystems.xml").findall(".//numberingSystem"):
        code = row.get("id")
        if code not in selected:
            continue
        if not names.get(code):
            raise ValueError(f"missing CLDR numbering name: {code}")
        fields = dict(row.attrib)
        if fields["type"] == "numeric" and len(fields.get("digits", "")) != 10:
            raise ValueError(f"invalid decimal digit set: {code}")
        if fields["type"] not in {"numeric", "algorithmic"}:
            raise ValueError(f"invalid numbering system kind: {code}")
        systems[code] = (names[code], fields)
    if systems.keys() != selected:
        raise ValueError("numbering preferences reference unknown systems")
    return systems, preferences


def build_catalog(data, policy):
    systems, _ = records(data, policy)
    return validate(Catalog(CatalogSpec("numbering-systems", "NumberingSystem", "code", "cldr",
                                      icu_names=False, label_source="Cldr",
                                      description="Common CLDR numbering systems with CLDR English names."), tuple(
        Entry(code, variant_name(code), name, f"iso-numbering-systems-{code}", (), {})
        for code, (name, _) in sorted(systems.items()))))


def render(data, policy, catalog):
    systems, preferences = records(data, policy)
    kinds = {"numeric": "Numeric", "algorithmic": "Algorithmic"}
    result = GENERATED + "use crate::{NumberingSystem, NumberingPreferences};\n"
    result += metadata_enum("NumberingSystemKind", kinds, "CLDR's distinction between decimal digits and rule-based numerals.")
    result += "impl NumberingSystem {\n"
    result += value_table("kind", "NumberingSystemKind", [f"NumberingSystemKind::{kinds[systems[e.code][1]['type']]}" for e in catalog.entries])
    result += "    /// Ten decimal digits in value order; algorithmic systems use rules instead.\n"
    result += string_table("digits", [systems[e.code][1].get("digits") for e in catalog.entries], optional=True)
    result += "    /// The upstream CLDR RBNF ruleset identifier for algorithmic systems.\n"
    result += string_table("rules", [systems[e.code][1].get("rules") for e in catalog.entries], optional=True)
    result += "}\n"
    variants = {e.code: e.variant for e in catalog.entries}
    rows = []
    for locale, values in sorted(preferences.items()):
        fields = ", ".join(f"{field}: NumberingSystem::{variants[code]}" for field, code in
                           zip(("default", "native", "traditional", "finance"), values))
        rows.append(f"({literal(locale)}, NumberingPreferences {{ {fields} }})")
    result += f"pub(super) const PREFERENCES: &[(&str, NumberingPreferences)] = &[{', '.join(rows)}];\n"
    return result
