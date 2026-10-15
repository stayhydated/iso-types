"""Read released UNECE trade code lists and explicit CLDR equivalences."""

import hashlib
import json
import re
import xml.etree.ElementTree as ET
from zipfile import ZipFile

from .catalog import Catalog, CatalogSpec, Entry
from .output import GENERATED
from .registries import validate
from .render import value_table, slice_table


def workbook_rows(path, sheet):
    """Read the sparse cells in the released XLSX without a runtime dependency."""
    namespace = {"s": "http://schemas.openxmlformats.org/spreadsheetml/2006/main"}
    with ZipFile(path) as archive:
        strings = ["".join(node.itertext()) for node in ET.fromstring(archive.read("xl/sharedStrings.xml")).findall("s:si", namespace)]
        rows = ET.fromstring(archive.read(f"xl/worksheets/sheet{sheet}.xml")).findall("s:sheetData/s:row", namespace)
        for row in rows:
            result = {}
            for cell in row.findall("s:c", namespace):
                value = cell.findtext("s:v", namespaces=namespace)
                if value is not None:
                    column = re.sub("[0-9]", "", cell.get("r"))
                    result[column] = strings[int(value)] if cell.get("t") == "s" else value
            yield result


def unit_records(data):
    result = {}
    for row in list(workbook_rows(data / "registries/rec20.xlsx", 3))[1:]:
        code = row.get("B", "")
        if not code:
            continue
        if not re.fullmatch("[A-Z0-9]{2,3}", code) or code in result:
            raise ValueError(f"invalid or duplicate Rec 20 code: {code}")
        status = row.get("A", "")
        if status not in {"", "+", "#", "¦", "D", "X"}:
            raise ValueError(f"unknown Rec 20 status: {code}/{status}")
        if status in {"D", "X"}:
            continue
        result[code] = {"name": row["C"].strip(), **{
            key: row[column] for key, column in (("description", "D"), ("level", "E"), ("symbol", "F"), ("conversion_factor", "G")) if row.get(column)}}
    return result


def packaging_records(data):
    root = data / "registries"
    snapshot = json.loads((root / "rec21.json").read_text(encoding="utf-8"))
    if snapshot["source_sha256"] != hashlib.sha256((root / "rec21.xls").read_bytes()).hexdigest():
        raise ValueError("packaging workbook snapshot is stale; run tools/import_packaging.py")
    rows = next(s["rows"] for s in snapshot["sheets"] if s["name"] == "Annex V and VI")
    if rows[2][:3] != ["Status", "Code", "Name"]:
        raise ValueError("unexpected Rec 21 workbook columns")
    result = {}
    for status, code, name, description, numeric in rows[3:]:
        if not code:
            continue
        code = str(int(code)) if isinstance(code, (int, float)) else code
        if not re.fullmatch("[A-Z0-9]{2}", code) or code in result:
            raise ValueError(f"invalid or duplicate Rec 21 code: {code}")
        if status not in {"", "+", "#", "¦", "D", "X"}:
            raise ValueError(f"unknown Rec 21 status: {code}/{status}")
        if status in {"D", "X"}:
            continue
        numeric = str(int(numeric)) if isinstance(numeric, (int, float)) else numeric
        result[code] = {"name": name.strip(), **({"description": description} if description else {}),
                        **({"numeric_category": numeric} if numeric else {})}
    return result


def build_catalogs(data):
    result = []
    packages = packaging_records(data)
    units = unit_records(data)
    # Rec 20's published guidance reserves X00..XZZ for Rec 21 package counts.
    for code, row in packages.items():
        key = "X" + code
        if key in units:
            raise ValueError(f"package unit collides with a Rec 20 code: {key}")
        units[key] = {"name": row["name"], "packaging_code": code}
    for standard, name, records, description in (
        ("trade-units", "TradeUnit", units, "Current UNECE Recommendation 20 unit codes, including X-prefixed Recommendation 21 package counts."),
        ("packaging", "PackagingType", packages, "Current UNECE Recommendation 21 package codes with the publisher's English names."),
    ):
        result.append(validate(Catalog(CatalogSpec(standard, name, "code", "unece", icu_names=False,
            label_source="Unece", description=description), tuple(
            Entry(code, ("Code" if code[0].isdigit() else "") + code.title(), row["name"],
                  f"iso-{standard}-{code}", (), {k: v for k, v in row.items() if k != "name"})
            for code, row in sorted(records.items())))))
    return tuple(result)


def render(data, catalogs):
    by_id = {c.spec.standard: c for c in catalogs}
    trade, units = by_id["trade-units"], by_id["units"]
    selected_trade = {e.code: e for e in trade.entries}
    selected_units = {e.code: e for e in units.entries}
    crosswalk = json.loads((data / "registries/unit-crosswalk.json").read_text(encoding="utf-8"))["unece_to_cldr"]
    if not crosswalk.keys() <= selected_trade.keys() or not set(crosswalk.values()) <= selected_units.keys():
        raise ValueError("trade unit equivalence references an unavailable code")
    output = GENERATED + '#[cfg(all(feature = "trade-units", feature = "units"))]\nimpl crate::TradeUnit {\n'
    output += "    /// Explicit semantic equivalence to a selected CLDR unit; conversion uses ICU4X.\n"
    output += value_table("catalog_unit", "Option<crate::MeasurementUnit>", [
        f"Some(crate::MeasurementUnit::{selected_units[crosswalk[e.code]].variant})" if e.code in crosswalk else "None" for e in trade.entries])
    output += '}\n#[cfg(all(feature = "trade-units", feature = "units"))]\nimpl crate::MeasurementUnit {\n'
    output += slice_table("trade_codes", "crate::TradeUnit", [tuple(e for e in trade.entries if crosswalk.get(e.code) == unit.code) for unit in units.entries])
    output += '}\n#[cfg(all(feature = "trade-units", feature = "packaging"))]\nimpl crate::TradeUnit {\n'
    packages = {e.code: e for e in by_id["packaging"].entries}
    output += value_table("packaging_type", "Option<crate::PackagingType>", [
        f"Some(crate::PackagingType::{packages[e.metadata['packaging_code']].variant})" if "packaging_code" in e.metadata else "None" for e in trade.entries])
    output += '}\n#[cfg(all(feature = "trade-units", feature = "packaging"))]\nimpl crate::PackagingType {\n'
    output += value_table("trade_unit", "crate::TradeUnit", [f"crate::TradeUnit::{selected_trade['X' + e.code].variant}" for e in by_id["packaging"].entries])
    return output + "}\n"
