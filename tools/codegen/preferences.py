"""Resolve CLDR territory defaults without conflating them with user overrides."""

import xml.etree.ElementTree as ET

from .output import GENERATED
from .render import value_table


def indexed(rows, regions_key, value_key):
    result = {}
    for row in rows:
        if row.get("alt"):
            continue
        for code in row.get(regions_key).split():
            if code in result:
                raise ValueError(f"duplicate CLDR preference: {code}/{value_key}")
            result[code] = row.get(value_key)
    if "001" not in result:
        raise ValueError(f"missing CLDR world default: {value_key}")
    return result


def regional_rows(data, countries):
    source = ET.parse(data / "current/supplementalData.xml")
    mappings = {
        tag: indexed(source.findall(f".//weekData/{tag}"), "territories", key)
        for tag, key in [("firstDay", "day"), ("weekendStart", "day"), ("weekendEnd", "day"), ("minDays", "count")]
    }
    mappings["hourCycle"] = indexed(source.findall(".//timeData/hours"), "regions", "preferred")
    mappings["calendar"] = indexed(source.findall(".//calendarPreferenceData/calendarPreference"), "territories", "ordering")
    mappings["measurement"] = indexed([r for r in source.findall('.//measurementData/measurementSystem') if r.get('category') is None],
                                     "territories", "type")
    mappings["paperSize"] = indexed(source.findall(".//measurementData/paperSize"), "territories", "type")
    temperature = {c: r.get("type") for r in source.findall('.//measurementData/measurementSystem[@category="temperature"]')
                   for c in r.get("territories").split()}
    rows = []
    for entry in countries.entries:
        values = {key: mapping.get(entry.code, mapping["001"]) for key, mapping in mappings.items()}
        values["temperature"] = temperature.get(entry.code, values["measurement"])
        rows.append(values)
    return rows


def calendar(code):
    if code.startswith("islamic"):
        suffix = code.removeprefix("islamic").removeprefix("-")
        value = f"Some(HijriCalendarAlgorithm::{ {'umalqura':'Umalqura', 'civil':'Civil', 'tbla':'Tbla', 'rgsa':'Rgsa'}[suffix]})" if suffix else "None"
        return f"CalendarAlgorithm::Hijri({value})"
    variants = {"gregorian":"Gregory", "buddhist":"Buddhist", "chinese":"Chinese", "coptic":"Coptic", "dangi":"Dangi",
                "ethiopic":"Ethiopic", "hebrew":"Hebrew", "indian":"Indian", "japanese":"Japanese", "persian":"Persian", "roc":"Roc"}
    return f"CalendarAlgorithm::{variants[code]}"


def render_regional(data, countries):
    days = dict(zip("mon tue wed thu fri sat sun".split(), "Monday Tuesday Wednesday Thursday Friday Saturday Sunday".split()))
    systems = {"metric":"Metric", "US":"USSystem", "UK":"UKSystem"}
    values = []
    for r in regional_rows(data, countries):
        values.append("RegionalPreferences { week: WeekPreferences { "
                      + ", ".join(f"{name}: crate::Iso8601Weekday::{days[r[key]]}" for name, key in
                                  [("first_weekday","firstDay"),("weekend_start","weekendStart"),("weekend_end","weekendEnd")])
                      + f", min_week_days: {r['minDays']} }}, hour_cycle: HourCycle::{ {'h':'H12','H':'H23','K':'H11'}[r['hourCycle']]}, "
                      + "preferred_calendars: &[" + ", ".join(calendar(c) for c in r["calendar"].split()) + "], "
                      + f"measurement_system: MeasurementSystem::{systems[r['measurement']]}, "
                      + f"temperature_measurement_system: MeasurementSystem::{systems[r['temperature']]}, "
                      + f"paper_size: PaperSize::{ {'A4':'A4','US-Letter':'UsLetter'}[r['paperSize']]} }}")
    return GENERATED + "use crate::regional::*;\n" + '''impl crate::Iso3166_1 {
    /// CLDR territory defaults; explicit locale and application preferences take precedence.
''' + value_table("regional_preferences", "RegionalPreferences", values) + "}\n"
