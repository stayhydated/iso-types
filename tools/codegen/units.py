"""Select everyday CLDR units with authoritative English display names."""

from pathlib import Path
import xml.etree.ElementTree as ET

from .catalog import Catalog, CatalogSpec, Entry
from .policy import CurrentData
from .relations import variant_name
from .unit_preferences import required_units


COMMON_UNITS = tuple("""
meter kilometer centimeter millimeter micrometer nanometer foot inch yard mile
square-meter square-kilometer square-centimeter square-foot square-inch acre hectare
liter milliliter cubic-meter cubic-centimeter gallon gallon-imperial quart pint cup
fluid-ounce tablespoon teaspoon gram kilogram milligram tonne ounce pound stone
second millisecond microsecond nanosecond minute hour day week celsius fahrenheit kelvin
meter-per-second kilometer-per-hour mile-per-hour bit byte kilobit kilobyte megabit
megabyte gigabit gigabyte terabit terabyte watt kilowatt megawatt joule kilojoule
kilowatt-hour calorie kilocalorie pascal hectopascal kilopascal bar millibar
pound-force-per-square-inch hertz kilohertz megahertz gigahertz degree radian
newton ampere volt ohm
liter-per-100-kilometer mile-per-gallon mile-per-gallon-imperial
""".split())


def build_units(data: Path, policy: CurrentData) -> Catalog:
    regular = {long_id.partition("-")[2]: long_id for long_id in policy.validity("unit", {"regular"})}
    names = {row.get("type"): row.findtext("displayName") for row in
             ET.parse(data / "current/units-en.xml").findall('.//unitLength[@type="long"]/unit')}
    entries = []
    if len(set(COMMON_UNITS)) != len(COMMON_UNITS):
        raise ValueError("duplicate selected measurement unit")
    for code in sorted(set(COMMON_UNITS) | (required_units(data) & regular.keys())):
        long_id = regular.get(code)
        if long_id is None:
            raise ValueError(f"selected unit is outside CLDR regular validity: {code}")
        name = names.get(long_id)
        if not name:
            raise ValueError(f"CLDR English unit name is unavailable: {code}")
        entries.append(Entry(code, variant_name(code), name, f"iso-units-{code}", (),
                             {"category": long_id.partition("-")[0]}))
    return Catalog(CatalogSpec("units", "MeasurementUnit", "code", "units", icu_names=False, label_source="Cldr"), tuple(entries))
