"""Snapshot the complete Rec 21 workbook with xlrd 2.0.2 (import-time only).

Run: uv run --with xlrd==2.0.2 --no-project python tools/import_packaging.py
"""

import hashlib
import json
from pathlib import Path

import xlrd


def main():
    if xlrd.__version__ != "2.0.2":
        raise ValueError("the workbook snapshot requires xlrd 2.0.2")
    root = Path(__file__).resolve().parent.parent / "data/registries"
    source = root / "rec21.xls"
    workbook = xlrd.open_workbook(source)
    snapshot = {
        "source": source.name,
        "source_sha256": hashlib.sha256(source.read_bytes()).hexdigest(),
        "extractor": "xlrd 2.0.2",
        "sheets": [{"name": sheet.name, "rows": [sheet.row_values(i) for i in range(sheet.nrows)]}
                   for sheet in workbook.sheets()],
    }
    output = root / "rec21.json"
    output.write_text(json.dumps(snapshot, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    provenance = json.loads((root / "provenance.json").read_text(encoding="utf-8"))
    provenance["files"][output.name] = {
        "url": provenance["files"][source.name]["url"],
        "sha256": hashlib.sha256(output.read_bytes()).hexdigest(),
        "derived_from": source.name,
        "extractor": snapshot["extractor"],
        "license": "CC-BY-4.0",
    }
    (root / "provenance.json").write_text(json.dumps(provenance, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
