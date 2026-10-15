"""Preserve the Library of Congress language-group coding hierarchy."""

from html.parser import HTMLParser
import re

from .catalog import Catalog, CatalogSpec, Entry
from .output import GENERATED
from .registries import validate
from .render import slice_table, value_table


class TableParser(HTMLParser):
    def __init__(self):
        super().__init__()
        self.rows = []
        self.row = None
        self.cell = None

    def handle_starttag(self, tag, attrs):
        if tag == "tr":
            self.row = []
        elif tag in {"td", "th"}:
            self.cell = []

    def handle_data(self, data):
        if self.cell is not None:
            self.cell.append(data)

    def handle_endtag(self, tag):
        if tag in {"td", "th"} and self.cell is not None:
            if self.row is not None:
                self.row.append("".join(self.cell).strip())
            self.cell = None
        elif tag == "tr" and self.row is not None:
            self.rows.append(self.row)
            self.row = None


def records(data):
    parser = TableParser()
    parser.feed((data / "registries/language-groups.html").read_text(encoding="utf-8"))
    result = {}
    for row in parser.rows:
        if len(row) != 6 or not re.fullmatch("[a-z]{3}", row[0]):
            continue
        code, name, _, _, raw_hierarchy, _ = row
        hierarchy = tuple(part.strip() for part in raw_hierarchy.split(":"))
        if code in result or not name or not hierarchy or hierarchy[-1] != code or len(set(hierarchy)) != len(hierarchy):
            raise ValueError(f"invalid language group: {code}")
        result[code] = (name, hierarchy)
    if not result:
        raise ValueError("missing language-group table")
    for code, (_, chain) in result.items():
        for index, ancestor in enumerate(chain):
            if ancestor not in result or result[ancestor][1] != chain[:index + 1]:
                raise ValueError(f"inconsistent language group hierarchy: {code}/{ancestor}")
    return result


def build_catalog(data):
    return validate(Catalog(CatalogSpec("639-5", "Iso6395", "code", "loc", icu_names=False,
        label_source="LibraryOfCongress", description="ISO 639 Set 5 language groups; English names and coding hierarchy are owned by the Library of Congress."), tuple(
        Entry(code, code.title(), name, f"iso-639-5-{code}", (), {})
        for code, (name, _) in sorted(records(data).items()))))


def render(data, catalog):
    source = records(data)
    by_code = {e.code: e for e in catalog.entries}
    kind = "crate::Iso6395"
    output = GENERATED + f"impl {kind} {{\n"
    output += "    /// Direct parent in the published coding hierarchy.\n"
    output += value_table("parent", f"Option<{kind}>", [
        f"Some({kind}::{by_code[source[e.code][1][-2]].variant})" if len(source[e.code][1]) > 1 else "None" for e in catalog.entries])
    output += "    /// Direct child groups, in canonical code order.\n"
    output += slice_table("children", kind, [tuple(child for child in catalog.entries
        if len(source[child.code][1]) > 1 and source[child.code][1][-2] == e.code) for e in catalog.entries])
    output += "    /// Published ancestors from the broadest group to the direct parent.\n"
    output += slice_table("ancestors", kind, [tuple(by_code[c] for c in source[e.code][1][:-1]) for e in catalog.entries])
    return output + "}\n"
