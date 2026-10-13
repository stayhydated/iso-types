"""Preserve CLDR usage-specific ranked and mixed-unit preferences."""

import xml.etree.ElementTree as ET

from .catalog import Catalog, CatalogSpec, Entry
from .output import GENERATED
from .relations import variant_name
from .rust import literal

PREFERENCE_CATEGORIES = {"area", "consumption", "duration", "energy", "length", "mass", "power", "pressure", "speed", "temperature", "volume"}
USAGE_NAMES = {
    "default":"Default", "geograph":"Geographic", "land":"Land", "floor":"Floor",
    "vehicle-fuel":"Vehicle fuel", "media":"Media", "food":"Food", "focal-length":"Focal length",
    "person":"Person", "person-height":"Person height", "rainfall":"Rainfall", "road":"Road",
    "snowfall":"Snowfall", "vehicle":"Vehicle", "visiblty":"Visibility", "engine":"Engine",
    "baromtrc":"Barometric", "wind":"Wind", "weather":"Weather", "fluid":"Fluid", "oil":"Oil",
}


def groups(data):
    return [g for g in ET.parse(data / "current/unitPreferences.xml").findall(".//unitPreferences")
            if g.get("category") in PREFERENCE_CATEGORIES]


def required_units(data):
    return {unit.strip() for group in groups(data) for row in group
            for unit in row.text.strip().split("-and-")}


def build_usage_catalog(data):
    entries = []
    for group in groups(data):
        category, usage = group.get("category"), group.get("usage")
        code = f"{category}-{usage}"
        name = f"{USAGE_NAMES[usage]} {category}"
        entries.append(Entry(code, variant_name(code), name, f"iso-unit-preferences-{code}", (),
                             {"category": category, "usage": usage}))
    if len({e.code for e in entries}) != len(entries):
        raise ValueError("duplicate CLDR unit usage")
    return Catalog(CatalogSpec("unit-preferences", "UnitUsage", "code", "cldr", False), tuple(sorted(entries, key=lambda e:e.code)))


def preference_rows(group):
    regions = {}
    for row in group:
        units = tuple(row.text.strip().split("-and-"))
        threshold = float(row.get("geq", "1"))
        if threshold < 0:
            raise ValueError("negative CLDR unit preference threshold")
        value = (units, threshold, row.get("skeleton"))
        for country in row.get("regions").split():
            regions.setdefault(country, []).append(value)
    if "001" not in regions:
        raise ValueError("missing CLDR unit preference world default")
    return regions


def render(data, catalogs):
    by_id = {c.spec.standard:c for c in catalogs}
    units = {e.code:e for e in by_id["units"].entries}
    usage = by_id["unit-preferences"]
    countries = by_id["3166-1"]
    definitions, tables = [], []
    by_code = {f"{g.get('category')}-{g.get('usage')}":g for g in groups(data)}
    index = 0
    for entry in usage.entries:
        regions = preference_rows(by_code[entry.code])
        names = {}
        for country, rows in sorted(regions.items()):
            values = []
            for codes, threshold, skeleton in rows:
                variants = ", ".join(f"super::PreferredUnit {{ code: {literal(c)}, catalog_unit: "
                                     + (f"Some(crate::MeasurementUnit::{units[c].variant})" if c in units else "None") + " }" for c in codes)
                skeleton = f"Some({literal(skeleton)})" if skeleton else "None"
                values.append(f"UnitPreference {{ units: &[{variants}], threshold: {threshold}, skeleton: {skeleton} }}")
            name = f"P{index}"
            index += 1
            definitions.append(f"const {name}: &[UnitPreference] = &[{', '.join(values)}];")
            names[country] = name
        tables.append("&[" + ", ".join(names.get(e.code, names["001"]) for e in countries.entries) + "]")
    return GENERATED + "use super::UnitPreference;\n" + "\n".join(definitions) + f'''
impl crate::UnitUsage {{
    /// Ranked CLDR choices for this usage and country, including mixed units and formatting skeletons.
    pub const fn preferences(self, country: crate::Iso3166_1) -> &'static [UnitPreference] {{
        const TABLES: &[&[&[UnitPreference]]] = &[{', '.join(tables)}];
        TABLES[self as usize][country as usize]
    }}
}}
impl crate::Iso3166_1 {{
    pub const fn unit_preferences(self, usage: crate::UnitUsage) -> &'static [UnitPreference] {{ usage.preferences(self) }}
}}
'''
