"""Validate registered national IBAN formats from the complete Swift TXT file."""

import csv
from datetime import datetime
import io
import json
import re

from .output import GENERATED
from .render import value_table
from .rust import literal


def components(pattern):
    parts = [(int(length), kind) for length, kind in re.findall(r"([0-9]+)!([nac])", pattern)]
    if not parts or "".join(f"{n}!{k}" for n, k in parts) != pattern or any(n == 0 for n, _ in parts):
        raise ValueError(f"invalid fixed-width IBAN structure: {pattern}")
    return parts


def identifier_range(raw, length):
    if raw in {"", "N/A"}:
        return None
    match = re.fullmatch(r"([0-9]+)-([0-9]+)", raw)
    if not match:
        raise ValueError(f"invalid IBAN identifier positions: {raw}")
    start, end = map(int, match.groups())
    if not 1 <= start <= end <= length:
        raise ValueError(f"IBAN identifier positions outside BBAN: {raw}")
    return start - 1, end


def records(data, countries):
    as_of = json.loads((data / "registries/provenance.json").read_text(encoding="utf-8"))["as_of"]
    raw = (data / "registries/iban-registry.txt").read_bytes().decode("cp1252")
    rows = list(csv.reader(io.StringIO(raw), delimiter="\t"))
    wanted = {
        "IBAN prefix country code (ISO 3166)", "Country code includes other countries/territories",
        "BBAN structure", "BBAN length", "IBAN structure", "IBAN length", "Effective date",
        "Bank identifier position within the BBAN", "Branch identifier position within the BBAN",
        "IBAN electronic format example", "IBAN print format example",
    }
    fields = {}
    for row in rows:
        if row and row[0] in wanted:
            if row[0] in fields:
                raise ValueError(f"duplicate IBAN data element: {row[0]}")
            fields[row[0]] = row[1:]
    if fields.keys() != wanted or len({len(row) for row in fields.values()}) != 1:
        raise ValueError("incomplete or ragged IBAN registry columns")
    result = {}
    for index, code in enumerate(fields["IBAN prefix country code (ISO 3166)"]):
        if code not in countries:
            continue
        row = {key: values[index] for key, values in fields.items()}
        if code in result:
            raise ValueError(f"duplicate IBAN country prefix: {code}")
        bban = row["BBAN structure"]
        length = int(row["IBAN length"])
        bban_length = int(row["BBAN length"])
        if not 5 <= length <= 34 or length != bban_length + 4 or sum(n for n, _ in components(bban)) != bban_length:
            raise ValueError(f"invalid registered IBAN lengths: {code}")
        full = row["IBAN structure"]
        if not full.startswith(code + "2!n") or sum(n for n, _ in components(full[2:])) != length - 2:
            raise ValueError(f"invalid full IBAN structure: {code}")
        # Segment boundaries can differ (CZ: 4!n16!n vs 4!n6!n10!n),
        # while the per-position character classes must agree.
        expanded = lambda pattern: "".join(kind * n for n, kind in components(pattern))
        if expanded(full[5:]) != expanded(bban):
            raise ValueError(f"conflicting IBAN/BBAN character classes: {code}")
        effective = datetime.strptime(row["Effective date"], "%b-%y").date().isoformat()
        if effective > as_of:
            continue
        countries_for_format = {code}
        raw_others = row["Country code includes other countries/territories"]
        if raw_others != "N/A":
            for token in raw_others.split(","):
                match = re.fullmatch(r"\s*([A-Z]{2})(?:\s+\([^)]*\))?\s*", token)
                if not match or match[1] not in countries:
                    raise ValueError(f"unavailable IBAN territory: {code}/{token}")
                countries_for_format.add(match[1])
        result[code] = {
            "bban_structure": bban, "iban_structure": full, "iban_length": length,
            "components": components(bban), "countries": tuple(sorted(countries_for_format)),
            "bank_range": identifier_range(row["Bank identifier position within the BBAN"], bban_length),
            "branch_range": identifier_range(row["Branch identifier position within the BBAN"], bban_length),
            "effective_date": effective, "electronic_example": row["IBAN electronic format example"],
            "print_example": row["IBAN print format example"],
        }
    owners = {}
    for prefix, row in result.items():
        for country in row["countries"]:
            if country in owners:
                raise ValueError(f"ambiguous IBAN format for country: {country}")
            owners[country] = prefix
    return dict(sorted(result.items()))


def render(data, countries):
    country = {e.code: e for e in countries.entries}
    source = records(data, country)
    kinds = {"n": "Numeric", "a": "Alphabetic", "c": "Alphanumeric"}
    rows = []
    indexes = {}
    for index, (code, row) in enumerate(source.items()):
        fields = [f"prefix_country: crate::Iso3166_1::{country[code].variant}", f"iban_length: {row['iban_length']}"]
        for name in ("bban_structure", "iban_structure", "effective_date", "electronic_example", "print_example"):
            fields.append(f"{name}: {literal(row[name])}")
        fields.append("countries: &[" + ", ".join(f"crate::Iso3166_1::{country[c].variant}" for c in row["countries"]) + "]")
        fields.append("components: &[" + ", ".join(f"BbanComponent {{ length: {n}, kind: BbanCharacterKind::{kinds[k]} }}" for n, k in row["components"]) + "]")
        for field in ("bank_range", "branch_range"):
            pair = row[field]
            fields.append(f"{field}: " + (f"Some(({pair[0]}, {pair[1]}))" if pair else "None"))
        rows.append("IbanFormat { " + ", ".join(fields) + " }")
        indexes.update({c: index for c in row["countries"]})
    output = GENERATED + "use super::{IbanFormat, BbanComponent, BbanCharacterKind};\n"
    output += f"pub(super) const FORMATS: &[IbanFormat] = &[{', '.join(rows)}];\n"
    output += "impl crate::Iso3166_1 {\n    /// Registered IBAN prefix format, including explicitly associated territories.\n"
    output += value_table("iban_format", "Option<&'static IbanFormat>", [
        f"Some(&FORMATS[{indexes[e.code]}])" if e.code in indexes else "None" for e in countries.entries])
    return output + "}\n"
