"""Validate the current subdivision hierarchy and tender-currency relationships."""

from collections import defaultdict
from dataclasses import dataclass
import re

from .catalog import Catalog, Entry
from .policy import CurrentData


def variant_name(value: str) -> str:
    return "".join(word[0].upper() + word[1:] for word in re.findall(r"[A-Za-z0-9]+", value))


@dataclass(frozen=True)
class Relations:
    countries: Catalog
    subdivisions: Catalog
    currencies: Catalog
    kinds: dict[str, str]
    subdivision_countries: tuple[Entry, ...]
    parents: tuple[Entry | None, ...]
    country_subdivisions: dict[str, tuple[Entry, ...]]
    country_roots: dict[str, tuple[Entry, ...]]
    children: dict[str, tuple[Entry, ...]]
    country_currencies: dict[str, tuple[Entry, ...]]
    currency_countries: dict[str, tuple[Entry, ...]]
    as_of: str


def build_relations(catalogs: tuple[Catalog, ...], policy: CurrentData) -> Relations:
    catalogs_by_id = {catalog.spec.standard: catalog for catalog in catalogs}
    countries, subdivisions, currencies = (catalogs_by_id[standard] for standard in ("3166-1", "3166-2", "4217"))
    country_by_code = {entry.code: entry for entry in countries.entries}
    subdivision_by_code = {entry.code: entry for entry in subdivisions.entries}
    currency_by_code = {entry.code: entry for entry in currencies.entries}
    kinds = {name: variant_name(name) for name in sorted({entry.metadata["type"] for entry in subdivisions.entries})}
    if len(set(kinds.values())) != len(kinds):
        raise ValueError("subdivision kind names produce duplicate Rust variants")
    owners, parents = [], []
    by_country, roots, children = defaultdict(list), defaultdict(list), defaultdict(list)
    for entry in subdivisions.entries:
        country = entry.code.split("-", 1)[0]
        if country not in country_by_code:
            raise ValueError(f"subdivision country is unavailable: {entry.code}")
        owners.append(country_by_code[country])
        by_country[country].append(entry)
        parent_code = entry.metadata.get("parent")
        parent = subdivision_by_code.get(parent_code)
        if parent_code:
            if parent is None or parent.code.split("-", 1)[0] != country:
                raise ValueError(f"invalid subdivision parent: {entry.code} -> {parent_code}")
            children[parent_code].append(entry)
        else:
            roots[country].append(entry)
        parents.append(parent)
        seen = {entry.code}
        while parent is not None:
            if parent.code in seen:
                raise ValueError(f"cyclic subdivision hierarchy: {entry.code}")
            seen.add(parent.code)
            parent_code = parent.metadata.get("parent")
            if parent_code and parent_code not in subdivision_by_code:
                raise ValueError(f"subdivision parent is unavailable: {parent.code}")
            parent = subdivision_by_code.get(parent_code)
    country_currencies, currency_countries = {}, defaultdict(list)
    for country in countries.entries:
        selected = tuple(currency_by_code[code] for code in policy.currency_regions.get(country.code, ())
                         if code in currency_by_code)
        country_currencies[country.code] = selected
        for currency in selected:
            currency_countries[currency.code].append(country)
    return Relations(countries, subdivisions, currencies, kinds, tuple(owners), tuple(parents),
                     {code: tuple(entries) for code, entries in by_country.items()},
                     {code: tuple(entries) for code, entries in roots.items()},
                     {code: tuple(entries) for code, entries in children.items()},
                     country_currencies, {code: tuple(entries) for code, entries in currency_countries.items()},
                     policy.as_of)
