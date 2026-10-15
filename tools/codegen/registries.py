"""Verify pinned registry inputs and validate additional catalog models."""

from datetime import date
import hashlib
import json
from pathlib import Path

from .catalog import Catalog


def verify_sources(data: Path) -> dict:
    root = data / "registries"
    provenance = json.loads((root / "provenance.json").read_text(encoding="utf-8"))
    date.fromisoformat(provenance["as_of"])
    for name, source in provenance["files"].items():
        if hashlib.sha256((root / name).read_bytes()).hexdigest() != source["sha256"]:
            raise ValueError(f"registry source checksum mismatch: {name}")
    return provenance


def validate(catalog: Catalog) -> Catalog:
    entries = catalog.entries
    if not entries or tuple(sorted(entries, key=lambda e: e.code)) != entries:
        raise ValueError(f"empty or unsorted registry catalog: {catalog.spec.standard}")
    for field in ("code", "variant", "message_id"):
        if len({getattr(entry, field) for entry in entries}) != len(entries):
            raise ValueError(f"duplicate {field}: {catalog.spec.standard}")
    for entry in entries:
        if not entry.name or any(c in entry.name for c in "\r\n{}"):
            raise ValueError(f"invalid registry label: {entry.code}")
    if len(entries) > 65535:
        raise ValueError(f"registry exceeds enum capacity: {catalog.spec.standard}")
    return catalog
