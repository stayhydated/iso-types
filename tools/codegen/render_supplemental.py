"""Render country navigation for UN regions and IANA zones."""

from collections import defaultdict

from .output import GENERATED
from .render import value_table, slice_table
from .supplemental import m49_rows, region_records, zone_records


def render(data, catalogs):
    by_id = {c.spec.standard: c for c in catalogs}
    countries, regions, zones = (by_id[s] for s in ("3166-1", "regions", "time-zones"))
    country = {e.code: e for e in countries.entries}
    region = {e.code: e for e in regions.entries}
    zone = {e.code: e for e in zones.entries}
    rows = {r["alpha_2"]: r for r in m49_rows(data)}
    chains, region_countries = {}, defaultdict(list)
    for entry in countries.entries:
        row = rows.get(entry.code)
        chain = tuple(region[row[f"{p}_code"]] for p in ("global", "region", "subregion", "intermediate")
                      if row and row[f"{p}_code"])
        chains[entry.code] = chain
        for r in chain:
            region_countries[r.code].append(entry)
    records = region_records(data)
    children = defaultdict(list)
    for entry in regions.entries:
        parent = records[entry.code]["parent"]
        if parent:
            children[parent].append(entry)
    owners, country_zones = {}, defaultdict(list)
    for code, values in zone_records(data).items():
        owners[code] = tuple(country[c] for c in values if c in country)
        for c in owners[code]:
            country_zones[c.code].append(zone[code])
    ct, rt, zt = "crate::Iso3166_1", "crate::GeographicRegion", "crate::TimeZoneId"
    return GENERATED + f'''
#[cfg(feature = "regions")]
impl {ct} {{
    /// UN M49 regions, ordered from World to the most specific recorded region.
{slice_table("geographic_regions", rt, [chains[e.code] for e in countries.entries])}
    /// The most specific UN M49 region in the pinned source.
{value_table("geographic_region", f"Option<{rt}>", [f"Some({rt}::{chains[e.code][-1].variant})" if chains[e.code] else "None" for e in countries.entries])}
}}
#[cfg(feature = "regions")]
impl {rt} {{
{value_table("parent", f"Option<{rt}>", [f"Some({rt}::{region[records[e.code]['parent']].variant})" if records[e.code]['parent'] else "None" for e in regions.entries])}
{slice_table("children", rt, [children[e.code] for e in regions.entries])}
{slice_table("countries", ct, [region_countries[e.code] for e in regions.entries])}
}}
#[cfg(feature = "time-zones")]
impl {ct} {{
    /// IANA location zones associated with this country in the pinned zone tables.
{slice_table("time_zones", zt, [country_zones[e.code] for e in countries.entries])}
}}
#[cfg(feature = "time-zones")]
impl {zt} {{
    /// Country associations combine IANA's zone.tab and zone1970.tab.
{slice_table("countries", ct, [owners[e.code] for e in zones.entries])}
}}
'''
