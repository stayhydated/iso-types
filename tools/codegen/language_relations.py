"""Render the selected language/script graph and CLDR territory associations."""

from collections import defaultdict
import xml.etree.ElementTree as ET

from .output import GENERATED
from .render import metadata_enum, slice_table, value_table
from .relations import variant_name
from .rust import literal


def macrolanguage_records(data):
    """Read explicit IANA membership without applying CLDR language aliases."""
    result = {}
    source = (data / "registries/language-subtag-registry.txt").read_text(encoding="utf-8")
    for block in source.split("%%"):
        fields = {}
        for line in block.splitlines():
            if not line or line.startswith(" "):
                continue
            key, separator, value = line.partition(": ")
            if separator:
                fields[key] = value
        if fields.get("Type") == "language" and "Deprecated" not in fields:
            code = fields["Subtag"]
            if code in result:
                raise ValueError(f"duplicate IANA language subtag: {code}")
            result[code] = fields.get("Macrolanguage")
    for code in result:
        visited = {code}
        parent = result[code]
        while parent is not None:
            if parent in visited:
                raise ValueError(f"cyclic macrolanguage relationship: {code}")
            visited.add(parent)
            # A deprecated parent remains an upstream identity; selection omits it.
            parent = result.get(parent)
    return result


def render_macrolanguages(data, catalogs):
    source = macrolanguage_records(data)
    result = ""
    for catalog in catalogs:
        if catalog.spec.standard not in {"639-1", "639-2", "639-3"}:
            continue
        # Exact ISO alpha-2 equivalence is safe; CLDR aliases can merge an
        # individual language into its macrolanguage and erase the relationship.
        key = lambda entry: entry.metadata.get("alpha_2", entry.code)
        selected = {key(entry): entry for entry in catalog.entries}
        parents = {entry.code: selected.get(source.get(key(entry))) for entry in catalog.entries}
        kind = f"crate::{catalog.spec.type_name}"
        result += f'#[cfg(feature = "{catalog.spec.feature}")]\nimpl {kind} {{\n'
        result += "    /// The explicit IANA macrolanguage, when selected in this catalog.\n"
        result += value_table("macrolanguage", f"Option<{kind}>", [
            f"Some({kind}::{parents[e.code].variant})" if parents[e.code] else "None" for e in catalog.entries])
        result += "    /// Direct individual-language members within the current/common catalog.\n"
        result += slice_table("members", kind, [tuple(child for child in catalog.entries
            if parents[child.code] == parent) for parent in catalog.entries])
        result += "}\n"
    return result


def render(data, policy, catalogs):
    by_id = {c.spec.standard:c for c in catalogs}
    countries = by_id["3166-1"]
    scripts = by_id["15924"]
    script = {e.code:e for e in scripts.entries}
    source = ET.parse(data / "current/supplementalData.xml")
    script_codes = defaultdict(set)
    for row in source.findall(".//languageData/language"):
        language = policy.language(row.get("type"))
        script_codes[language].update(c for c in row.get("scripts", "").split() if c in script)
    population = {}
    for row in source.findall(".//territoryInfo/territory"):
        records = []
        for usage in row.findall("languagePopulation"):
            raw = usage.get("type")
            language = policy.language(raw.split("_", 1)[0])
            percent = float(usage.get("populationPercent"))
            if not 0 <= percent <= 100:
                raise ValueError(f"invalid CLDR language population: {raw}")
            records.append((language, raw.replace("_", "-"), percent, usage.get("officialStatus")))
        population[row.get("type")] = records
    status_names = {s:variant_name(s) for s in ["de_facto_official", "official", "official_minority", "official_regional"]}
    output = GENERATED + metadata_enum("LanguageStatus", status_names, "CLDR's functional language status, distinct from a complete registry of legal designations.")
    output += '''
/// An estimated CLDR language association for a territory.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LanguageUsage {
    pub language: crate::Iso6391,
    /// The original CLDR language and optional script, in BCP47 spelling.
    pub locale: &'static str,
    /// Estimated functional population, including second-language users.
    pub population_percent: f64,
    pub status: Option<LanguageStatus>,
}
'''
    for standard, suffix in [("639-1", ""), ("639-2", "_6392"), ("639-3", "_6393")]:
        language_catalog = by_id[standard]
        by_language = {}
        for e in language_catalog.entries:
            language = policy.language(e.metadata.get("alpha_2", e.code))
            by_language.setdefault(language, []).append(e)
        language_countries = defaultdict(list)
        country_languages = {}
        for country in countries.entries:
            values = {e.code:e for lang, *_ in population.get(country.code, ()) for e in by_language.get(lang, ())}
            country_languages[country.code] = tuple(values[c] for c in sorted(values))
            for e in values.values():
                language_countries[e.code].append(country)
        lt = f"crate::{language_catalog.spec.type_name}"
        methods = slice_table("countries", "crate::Iso3166_1", [language_countries[e.code] for e in language_catalog.entries])
        methods += slice_table("scripts", "crate::Iso15924", [tuple(script[c] for c in sorted(script_codes[policy.language(e.metadata.get('alpha_2', e.code))])) for e in language_catalog.entries])
        output += f'#[cfg(feature = "{standard}")]\nimpl {lt} {{\n{methods}}}\n'
        output += f'#[cfg(feature = "{standard}")]\nimpl crate::Iso3166_1 {{\n'
        output += '    /// Languages recorded in CLDR territory data, filtered to the current/common catalog.\n'
        output += slice_table(f"languages{suffix}", lt, [country_languages[e.code] for e in countries.entries]) + "}\n"
        if standard == "639-1":
            values = []
            for country in countries.entries:
                records = []
                for lang, locale, percent, status in population.get(country.code, ()):
                    for e in by_language.get(lang, ()):
                        status_value = f"Some(LanguageStatus::{status_names[status]})" if status else "None"
                        records.append(f"LanguageUsage {{ language: {lt}::{e.variant}, locale: {literal(locale)}, population_percent: {percent}, status: {status_value} }}")
                values.append("&[" + ", ".join(records) + "]")
            output += "impl crate::Iso3166_1 {\n" + value_table("language_usage", "&'static [LanguageUsage]", values) + "}\n"
    language = by_id["639-1"]
    script_languages = {e.code:tuple(l for l in language.entries if e.code in script_codes[policy.language(l.code)]) for e in scripts.entries}
    output += "impl crate::Iso15924 {\n" + slice_table("languages", "crate::Iso6391", [script_languages[e.code] for e in scripts.entries]) + "}\n"
    return output + render_macrolanguages(data, catalogs)
