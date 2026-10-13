"""Select current, common ISO data from pinned CLDR and SIX metadata."""

from datetime import date
import hashlib
import json
from pathlib import Path
import xml.etree.ElementTree as ET

from .rust import literal


def expand_range(token: str) -> set[str]:
    """Expand CLDR's abbreviated, fixed-width alphanumeric ranges."""
    if "~" not in token:
        return {token}
    start, suffix = token.split("~", 1)
    if not start or not suffix or len(suffix) > len(start):
        raise ValueError(f"invalid CLDR range: {token}")
    end = start[:-len(suffix)] + suffix
    alphabets = [
        "0123456789" if char.isdigit() else
        "abcdefghijklmnopqrstuvwxyz" if char.islower() else
        "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
        for char in start
    ]
    if any(char not in alphabet for text in (start, end)
           for char, alphabet in zip(text, alphabets)) or start > end:
        raise ValueError(f"invalid CLDR range: {token}")
    result = set()
    current = start
    while True:
        result.add(current)
        if current == end:
            return result
        chars = list(current)
        for index in range(len(chars) - 1, -1, -1):
            alphabet = alphabets[index]
            next_index = alphabet.index(chars[index]) + 1
            if next_index < len(alphabet):
                chars[index] = alphabet[next_index]
                break
            chars[index] = alphabet[0]
        current = "".join(chars)


def property_rows(path: Path):
    for line in path.read_text(encoding="utf-8").splitlines():
        line = line.split("#", 1)[0].strip()
        if line:
            yield [field.strip() for field in line.split(";")]


class CurrentData:
    """Resolve source validity and accepted current ISO representations."""

    def __init__(self, data: Path) -> None:
        self.path = data / "current"
        self.provenance = json.loads((self.path / "provenance.json").read_text(encoding="utf-8"))
        for name, source in self.provenance["files"].items():
            if hashlib.sha256((self.path / name).read_bytes()).hexdigest() != source["sha256"]:
                raise ValueError(f"current-data source checksum mismatch: {name}")
        self.as_of = self.provenance["as_of"]
        date.fromisoformat(self.as_of)
        metadata = ET.parse(self.path / "supplementalMetadata.xml")
        self.language_aliases = {
            row.get("type"): (row.get("replacement"), row.get("reason"))
            for row in metadata.findall(".//languageAlias")
        }
        self.region_aliases = {
            row.get("type"): (row.get("replacement"), row.get("reason"))
            for row in metadata.findall(".//territoryAlias")
        }
        supplemental = ET.parse(self.path / "supplementalData.xml")
        self.scripts = self._recommended_scripts()
        self.languages = self._modern_languages(supplemental)
        self.regions = self.validity("region", {"regular", "macroregion"})
        self.subdivisions = self.validity("subdivision", {"regular"})
        self.currency_regions = self._currency_regions(supplemental)
        self.currencies = {code for currencies in self.currency_regions.values() for code in currencies}
        self.language_inputs = self.languages | {
            code for code, (_, reason) in self.language_aliases.items()
            if reason in {"overlong", "macrolanguage"} and self.language(code) in self.languages
        }
        self.region_inputs = self.regions | {
            code for code, (replacement, reason) in self.region_aliases.items()
            if reason == "overlong" and replacement in self.regions
        }
        iso_languages = json.loads((data / "iso/iso_639-3.json").read_text(encoding="utf-8"))
        self.living_languages = {row["alpha_3"] for row in iso_languages["639-3"] if row["type"] == "L"}

    def _recommended_scripts(self) -> set[str]:
        recommended = {fields[0] for fields in property_rows(self.path / "scriptMetadata.txt")
                       if fields[5] == "RECOMMENDED"}
        return recommended & self.validity("script", {"regular"})

    def _modern_languages(self, supplemental: ET.ElementTree) -> set[str]:
        locales = {fields[0].replace("_", "-") for fields in property_rows(self.path / "coverageLevels.txt")
                   if fields[1] == "modern"}
        # Explicit CLDR parents supplement ordinary subtag truncation.
        parents = supplemental.findall(".//parentLocales/parentLocale")
        while True:
            inherited = {child.replace("_", "-") for parent in parents
                         if parent.get("parent").replace("_", "-") in locales
                         for child in parent.get("locales").split()}
            if inherited <= locales:
                break
            locales |= inherited
        likely = {row.get("from"): row.get("to") for row in
                  ET.parse(self.path / "likelySubtags.xml").findall(".//likelySubtag")}
        result = set()
        for language in {locale.split("-")[0] for locale in locales}:
            subtags = likely.get(language, "").split("_")
            if len(subtags) > 1 and subtags[1] in self.scripts:
                result.add(language)
        return result

    def _currency_regions(self, supplemental: ET.ElementTree) -> dict[str, tuple[str, ...]]:
        current_iso = json.loads((self.path / "current_currencies.json").read_text(encoding="utf-8"))
        current = set(current_iso["codes"])
        return {
            region.get("iso3166"): tuple(sorted({row.get("iso4217") for row in region.findall("currency")
                if row.get("iso4217") in current and row.get("tender") != "false"
                and row.get("from", "0000-01-01") <= self.as_of
                and row.get("to", "9999-12-31") >= self.as_of}))
            for region in supplemental.findall(".//currencyData/region")
        }

    def validity(self, kind: str, statuses: set[str]) -> set[str]:
        return {code for row in ET.parse(self.path / f"{kind}.xml").findall(".//id")
                if row.get("idStatus") in statuses
                for token in (row.text or "").split() for code in expand_range(token)}

    def language(self, code: str) -> str | None:
        seen = set()
        while code in self.language_aliases:
            if code in seen:
                raise ValueError(f"cyclic CLDR language alias: {code}")
            seen.add(code)
            replacement, reason = self.language_aliases[code]
            if reason not in {"overlong", "macrolanguage"}:
                return None
            code = replacement.split("_")[0]
        return code

    def includes(self, standard: str, record: dict, code_key: str) -> bool:
        code = record[code_key]
        if standard in {"639-1", "639-2", "639-3"}:
            return record["alpha_3"] in self.living_languages and self.language(code) in self.languages
        if standard == "15924":
            return code in self.scripts
        if standard == "4217":
            return code in self.currencies
        if standard == "3166-1":
            return code in self.regions
        if standard == "3166-2":
            return code.replace("-", "").lower() in self.subdivisions
        return True

    def rust_locale_policy(self) -> str:
        def table(name: str, values: set[str]) -> str:
            return f"const {name}: &[&str] = &[" + ", ".join(literal(value) for value in sorted(values)) + "];\n"
        return (
            "// Generated by python3 -m tools.codegen from pinned current-data policy.\n"
            + table("LANGUAGES", self.language_inputs)
            + table("SCRIPTS", self.scripts)
            + table("REGIONS", self.region_inputs)
            + """pub(crate) fn includes(locale: &icu_locale::LanguageIdentifier) -> bool {
    LANGUAGES.binary_search(&locale.language.as_str()).is_ok()
        && locale.script.is_none_or(|script| SCRIPTS.binary_search(&script.as_str()).is_ok())
        && locale.region.is_none_or(|region| REGIONS.binary_search(&region.as_str()).is_ok())
        && locale.variants.is_empty()
}
"""
        )
