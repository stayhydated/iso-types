"""Preserve UN/LOCODE listings in a compact, separately packaged directory."""

from collections import defaultdict
import csv
import io
import re
from zipfile import ZipFile

from .output import GENERATED
from .rust import literal


STATUSES = {"AA", "AC", "AF", "AI", "AM", "AQ", "AS", "QQ", "RL", "RN", "RQ", "UR", "XX"}


def coordinate_minutes(text):
    match = re.fullmatch(r"([0-9]{2})([0-9]{2})([NS]) ([0-9]{3})([0-9]{2})([EW])", text)
    if not match:
        return None
    lat, lm, ns, lon, om, ew = match.groups()
    lat, lm, lon, om = map(int, (lat, lm, lon, om))
    if lm >= 60 or om >= 60 or lat > 90 or lon > 180 or (lat == 90 and lm) or (lon == 180 and om):
        return None
    return ((lat * 60 + lm) * (-1 if ns == "S" else 1),
            (lon * 60 + om) * (-1 if ew == "W" else 1))


def records(data, countries):
    result = defaultdict(list)
    with ZipFile(data / "registries/unlocode-2025-1.zip") as archive:
        members = sorted(name for name in archive.namelist() if "CodeListPart" in name and name.endswith(".csv"))
        if len(members) != 3:
            raise ValueError("UN/LOCODE release must contain three CSV parts")
        for member in members:
            rows = csv.reader(io.StringIO(archive.read(member).decode("utf-8-sig")))
            for row in rows:
                if len(row) != 12:
                    raise ValueError(f"invalid UN/LOCODE row in {member}")
                change, country, local, name, ascii_name, subdivision, functions, status, date, iata, coords, remarks = row
                if not local or country not in countries or change in {"=", "X"} or status == "XX":
                    continue
                code = country + local
                # Published identifiers include three entries with digits 0/1.
                if not re.fullmatch("[A-Z]{2}[A-Z0-9]{3}", code) or not name:
                    raise ValueError(f"invalid UN/LOCODE identifier or name: {code}")
                if status and status not in STATUSES:
                    raise ValueError(f"unknown UN/LOCODE status: {code}/{status}")
                if len(functions) != 8 or any(char not in ({"-", expected, "0"} if index == 0 else {"-", expected}) for index, (char, expected) in enumerate(zip(functions, "1234567B"))):
                    raise ValueError(f"invalid UN/LOCODE function mask: {code}/{functions}")
                if any(char in field for field in row for char in "\t\r\n"):
                    raise ValueError(f"UN/LOCODE field contains a TSV delimiter: {code}")
                result[code].append(row)
    if not result:
        raise ValueError("empty selected UN/LOCODE directory")
    return dict(sorted(result.items()))


def render(data, countries):
    source = records(data, {e.code for e in countries.entries})
    offsets = []
    chunks = []
    position = 0
    ranges = {}
    record_count = 0
    for index, (code, rows) in enumerate(source.items()):
        offsets.append(position)
        first, _ = ranges.get(code[:2], (index, index))
        ranges[code[:2]] = (first, index + 1)
        for row in rows:
            # All published alternatives stay together, in their original source order.
            text = "\t".join([code, *row[3:], row[0]]) + "\n"
            chunks.append(text)
            position += len(text.encode("utf-8"))
            record_count += 1
    offsets.append(position)
    if position >= 2**32:
        raise ValueError("UN/LOCODE byte offsets exceed u32")
    index = GENERATED + f"pub(super) const COUNT: usize = {len(source)};\n"
    index += f"pub(super) const RECORD_COUNT: usize = {record_count};\n"
    index += 'pub(super) const DIRECTORY: &str = include_str!("directory.tsv");\n'
    index += f"pub(super) const OFFSETS: &[u32] = &[{', '.join(map(str, offsets))}];\n"
    index += "pub(super) const COUNTRIES: &[(&str, u32, u32)] = &[" + ", ".join(
        f"({literal(country)}, {start}, {end})" for country, (start, end) in sorted(ranges.items())) + "];\n"
    return index, "".join(chunks)
