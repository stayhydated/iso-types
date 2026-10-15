"""Select ISO 216 A/B paper dimensions from the IANA IPP media registry."""

import re
import xml.etree.ElementTree as ET

from .catalog import Catalog, CatalogSpec, Entry
from .output import GENERATED
from .registries import validate
from .render import value_table


def records(data):
    namespace = {"i": "http://www.iana.org/assignments"}
    result = {}
    for row in ET.parse(data / "registries/ipp-registrations.xml").findall(".//i:record", namespace):
        if row.findtext("i:attribute", namespaces=namespace) != "media":
            continue
        name = row.findtext("i:value", default="", namespaces=namespace)
        match = re.fullmatch(r"iso_([ab](?:[0-9]|10))_([0-9]+)x([0-9]+)mm", name)
        if match:
            code, width, height = match.groups()
            value = (int(width), int(height), name)
            if code.upper() in result or not 0 < value[0] < value[1]:
                raise ValueError(f"invalid ISO paper dimensions: {name}")
            result[code.upper()] = value
    expected = {f"{series}{n}" for series in "AB" for n in range(11)}
    if result.keys() != expected:
        raise ValueError(f"incomplete ISO A/B media registry: {expected ^ result.keys()}")
    return result


def build_catalog(data):
    return validate(Catalog(CatalogSpec("216", "Iso216", "code", "iana", icu_names=False,
        description="ISO 216 A/B paper sizes; dimensions follow the pinned IANA IPP media registry."), tuple(
        Entry(code, code, f"{code} paper", f"iso-216-{code}", (), {})
        for code in sorted(records(data)))))


def render(data, catalog):
    rows = records(data)
    output = GENERATED + "impl crate::Iso216 {\n"
    for name, index in (("width_mm", 0), ("height_mm", 1)):
        output += "    /// The portrait dimension in whole millimeters.\n"
        output += value_table(name, "u16", [str(rows[e.code][index]) for e in catalog.entries])
    from .render import string_table
    output += "    /// The dimension-bearing media keyword in the IANA IPP registry.\n"
    output += string_table("pwg_name", [rows[e.code][2] for e in catalog.entries])
    return output + "}\n"
