"""Build finite geographic and time-zone catalogs from pinned source records."""

from collections import defaultdict
import json
from pathlib import Path

from .catalog import Catalog, CatalogSpec, Entry
from .relations import variant_name


def m49_rows(data: Path) -> list[dict]:
    rows = json.loads((data / "current/m49.json").read_text())["rows"]
    codes = [row["alpha_2"] for row in rows]
    if len(set(codes)) != len(codes):
        raise ValueError("duplicate UN M49 country")
    return rows


def region_records(data: Path) -> dict[str, dict]:
    records = {}
    for row in m49_rows(data):
        parent = None
        for prefix in ("global", "region", "subregion", "intermediate"):
            code = row[f"{prefix}_code"]
            if not code:
                continue
            record = {"name": row[f"{prefix}_name"], "parent": parent, "level": prefix}
            if code in records and records[code] != record:
                raise ValueError(f"inconsistent UN region: {code}")
            records[code] = record
            parent = code
    return dict(sorted(records.items()))


def zone_records(data: Path) -> dict[str, tuple[str, ...]]:
    owners = defaultdict(set)
    for filename in ("zone.tab", "zone1970.tab"):
        for line in (data / "current" / filename).read_text().splitlines():
            if not line or line.startswith("#"):
                continue
            countries, _, code, *_ = line.split("\t")
            owners[code].update(countries.split(","))
    owners["Etc/UTC"] = set()
    return {code: tuple(sorted(countries)) for code, countries in sorted(owners.items())}


def name_requests(data: Path) -> dict:
    return {"regions": list(region_records(data)), "time_zones": list(zone_records(data))}


def build_supplemental(data: Path, snapshot: dict) -> tuple[Catalog, ...]:
    regions = []
    for code, record in region_records(data).items():
        name = snapshot["regions"].get(code)
        if not name:
            raise ValueError(f"ICU region name is unavailable: {code}")
        regions.append(Entry(code, variant_name(name), name, f"iso-regions-{code}", (),
                             {"level": record["level"]}))
    zones = []
    for code in zone_records(data):
        record = snapshot["time_zones"][code]
        zones.append(Entry(code, "EtcUtc" if code == "Etc/UTC" else variant_name(code), record["name"],
                           "iso-time-zones-" + code.replace("/", "-").replace("_", "-"), ("UTC",) if code == "Etc/UTC" else (),
                           {"bcp47": record["bcp47"]}))
    return (
        Catalog(CatalogSpec("regions", "GeographicRegion", "code", "m49"), tuple(regions)),
        Catalog(CatalogSpec("time-zones", "TimeZoneId", "code", "iana"), tuple(zones)),
    )
