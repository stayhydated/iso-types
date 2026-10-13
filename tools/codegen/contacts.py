"""Generate calling-code relationships and postal-address schemas."""

from collections import defaultdict
import json
import re
import xml.etree.ElementTree as ET

from .catalog import Catalog, CatalogSpec, Entry
from .output import GENERATED
from .relations import variant_name
from .render import slice_table, value_table
from .rust import literal

FIELDS = {"A":("StreetAddress","Street address"), "C":("Locality","City / locality"),
          "D":("DependentLocality","Dependent locality"), "N":("Recipient","Recipient"),
          "O":("Organization","Organization"), "S":("AdministrativeArea","Administrative area"),
          "X":("SortingCode","Sorting code"), "Z":("PostalCode","Postal code"),
          "T":("LandmarkDescriptor","Landmark descriptor"), "F":("LandmarkAffix","Landmark affix"),
          "L":("LandmarkName","Landmark name")}
LABELS = {
    "recipient":"Recipient", "organization":"Organization", "street":"Street address", "sorting":"Sorting code",
    "area":"Area", "county":"County", "department":"Department", "district":"District",
    "do_si":"Province / metropolitan city", "emirate":"Emirate", "island":"Island", "oblast":"Oblast",
    "parish":"Parish", "prefecture":"Prefecture", "province":"Province", "state":"State",
    "city":"City", "post_town":"Post town", "suburb":"Suburb", "neighborhood":"Neighborhood", "townland":"Townland",
    "eircode":"Eircode", "pin":"PIN code", "postal":"Postal code", "zip":"ZIP code",
    "village_township":"Village / township", "landmark_descriptor":"Landmark descriptor",
    "landmark_affix":"Landmark affix", "landmark_name":"Landmark name",
}


def phone_rows(data):
    rows = ET.parse(data / "current/PhoneNumberMetadata.xml").findall(".//territory")
    for row in rows:
        code = row.get("countryCode")
        if not code.isdigit() or not 1 <= int(code) <= 999:
            raise ValueError(f"invalid telephone calling code: {code}")
    return rows


def build_catalogs(data):
    prefixes = sorted({"+" + row.get("countryCode") for row in phone_rows(data)})
    calling = Catalog(CatalogSpec("calling-codes", "CallingCode", "code", "libphonenumber", False), tuple(
        Entry(code, "C"+code[1:], f"Calling code {code}", "iso-calling-codes-"+code[1:], (), {}) for code in prefixes))
    fields = Catalog(CatalogSpec("address-fields", "AddressField", "code", "libaddressinput", False), tuple(
        Entry(code, variant, name, f"iso-address-fields-{code}", (), {}) for code,(variant,name) in sorted(FIELDS.items())))
    labels = Catalog(CatalogSpec("address-labels", "AddressFieldLabel", "code", "owned", False), tuple(
        Entry(code, variant_name(code), name, "iso-address-labels-"+code.replace("_","-"), (), {}) for code,name in sorted(LABELS.items())))
    return calling, fields, labels


def render_phones(data, catalogs):
    by_id = {c.spec.standard:c for c in catalogs}
    country, calling = by_id["3166-1"], by_id["calling-codes"]
    country_by_code = {e.code:e for e in country.entries}
    calling_by_code = {e.code:e for e in calling.entries}
    country_codes, owners = defaultdict(set), defaultdict(set)
    for row in phone_rows(data):
        c, prefix = row.get("id"), "+"+row.get("countryCode")
        if c in country_by_code:
            country_codes[c].add(prefix)
            owners[prefix].add(c)
    return GENERATED + "impl crate::Iso3166_1 {\n" + slice_table("calling_codes", "crate::CallingCode", [tuple(calling_by_code[c] for c in sorted(country_codes[e.code])) for e in country.entries]) + "}\n" + "impl crate::CallingCode {\n" + slice_table("countries", "crate::Iso3166_1", [tuple(country_by_code[c] for c in sorted(owners[e.code])) for e in calling.entries]) + value_table("number", "u16", [e.code[1:] for e in calling.entries]) + "}\n"


def format_fields(format):
    fields = []
    chars = iter(format)
    for char in chars:
        if char != "%":
            continue
        code = next(chars, None)
        if code == "n":
            continue
        if code not in FIELDS:
            raise ValueError(f"unknown address placeholder: {code}")
        if code not in fields:
            fields.append(code)
    return fields


def address_rows(data):
    live = json.loads((data / "current/addresses.json").read_text())
    # RegionDataConstants is a generated set of JSON literals joined by Java's `+`.
    # Parse that constrained data format, including its newer field and label metadata.
    text = (data / "current/RegionDataConstants.java").read_text()
    constants = {}
    pattern = r'map\.put\(\s*"([A-Z]{2})",\s*((?:"(?:\\.|[^"\\])*"\s*\+?\s*)+)\);'
    for match in re.finditer(pattern, text):
        parts = re.findall(r'"(?:\\.|[^"\\])*"', match[2])
        if match[1] in constants:
            raise ValueError("duplicate upstream address country")
        constants[match[1]] = json.loads("".join(json.loads(part) for part in parts))
    if "ZZ" not in constants or len(constants) < 249:
        raise ValueError("incomplete upstream address constants")
    source = {country: constants.get(country, {}) | (row or {})
              if row is not None or country in constants else None for country, row in live.items()}
    defaults = source["ZZ"]
    rows = {}
    for country, row in source.items():
        if country == "ZZ" or row is None:
            continue
        if row.get("key", country) != country:
            raise ValueError(f"address country differs from requested key: {country}")
        merged = defaults | row
        fields = format_fields(merged["fmt"])
        required = list(merged["require"])
        if len(set(required)) != len(required) or set(required) - set(fields):
            raise ValueError(f"invalid required address fields: {country}")
        merged["fields"] = fields
        for key in ("state_name_type", "locality_name_type", "sublocality_name_type", "zip_name_type"):
            if merged[key] not in LABELS:
                raise ValueError(f"unknown address label: {country}/{merged[key]}")
        rows[country] = merged
    return rows


def render_addresses(data, catalogs):
    countries = next(c for c in catalogs if c.spec.standard == "3166-1")
    rows = address_rows(data)
    def fields(codes):
        return "&[" + ", ".join(f"crate::AddressField::{FIELDS[c][0]}" for c in codes) + "]"
    def label(code):
        return f"crate::AddressFieldLabel::{variant_name(code)}"
    def optional(value):
        return f"Some({literal(value)})" if value else "None"
    values = []
    for country in countries.entries:
        row = rows.get(country.code)
        if row is None:
            values.append("None")
            continue
        examples = [v for v in row.get("zipex", "").split(",") if v]
        overrides = ["super::AddressLabelOverride { " + f"field: {literal(v['field'])}, language: {optional(v.get('lang'))}, "
                     + f"label: {optional(v.get('label'))}, message: {optional(v.get('message'))} }}"
                     for v in row.get("label_overrides", [])]
        languages = row.get("languages", "").split("~") if row.get("languages") else []
        values.append("Some(&AddressMetadata { " + f"format: {literal(row['fmt'])}, latin_format: {optional(row.get('lfmt'))}, "
                      + f"required_fields: {fields(row['require'])}, fields: {fields(row['fields'])}, "
                      + f"postal_code_pattern: {optional(row.get('zip'))}, postal_code_examples: &[{', '.join(literal(v) for v in examples)}], "
                      + f"label_overrides: &[{', '.join(overrides)}], default_language: {optional(row.get('lang'))}, languages: &[{', '.join(literal(v) for v in languages)}], "
                      + "labels: [" + ", ".join(label(row[k]) for k in ("state_name_type", "locality_name_type", "sublocality_name_type", "zip_name_type")) + "] })")
    return GENERATED + "use super::AddressMetadata;\nimpl crate::Iso3166_1 {\n" + value_table("address_metadata", "Option<&'static AddressMetadata>", values) + "}\n"
