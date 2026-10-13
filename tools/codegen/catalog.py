"""Validate source records and build the selected catalog model."""

from dataclasses import dataclass
import hashlib
import json
from pathlib import Path

from .policy import CurrentData


@dataclass(frozen=True)
class CatalogSpec:
    standard: str
    type_name: str
    code_key: str
    source: str
    icu_names: bool = True
    label_source: str | None = None

    @property
    def feature(self) -> str:
        if self.standard in {"address-fields", "address-labels"}:
            return "addresses"
        return "8601" if self.standard.startswith("8601-") else self.standard

    @property
    def module(self) -> str:
        if self.standard == "units":
            return "measurement_unit"
        if self.standard in {"regions", "time-zones", "unit-preferences", "calling-codes", "address-fields", "address-labels"}:
            return self.standard.replace("-", "_")
        return "iso_" + self.standard.replace("-", "_")

    @property
    def authority(self) -> str:
        return self.label_source or ("Icu4x" if self.icu_names else "Fluent")


SPECS = (
    CatalogSpec("639-1", "Iso6391", "alpha_2", "639-2"),
    CatalogSpec("639-2", "Iso6392", "alpha_3", "639-2"),
    CatalogSpec("639-3", "Iso6393", "alpha_3", "639-3"),
    CatalogSpec("3166-1", "Iso3166_1", "alpha_2", "3166-1"),
    CatalogSpec("3166-2", "Iso3166_2", "code", "3166-2", False),
    CatalogSpec("4217", "Iso4217", "alpha_3", "4217"),
    CatalogSpec("15924", "Iso15924", "alpha_4", "15924"),
    CatalogSpec("5218", "Iso5218", "code", "finite", False),
    CatalogSpec("8601-weekday", "Iso8601Weekday", "code", "finite"),
    CatalogSpec("8601-month", "Iso8601Month", "code", "finite"),
)

METADATA = {"alpha_2", "alpha_3", "alpha_4", "numeric", "common_name",
            "official_name", "type", "scope", "parent"}


@dataclass(frozen=True)
class Entry:
    code: str
    variant: str
    name: str
    message_id: str
    aliases: tuple[str, ...]
    metadata: dict[str, str]


@dataclass(frozen=True)
class Catalog:
    spec: CatalogSpec
    entries: tuple[Entry, ...]


def read_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def verify_sources(data: Path) -> None:
    provenance = read_json(data / "iso/provenance.json")
    for name, expected in provenance["sha256"].items():
        if hashlib.sha256((data / "iso" / name).read_bytes()).hexdigest() != expected:
            raise ValueError(f"ISO source checksum mismatch: {name}")


def read_ftl(path: Path) -> dict[str, str]:
    """The owned catalog format is a set of single-line literal messages."""
    if not path.is_file():
        return {}
    result = {}
    for number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        if not line or line.startswith("#"):
            continue
        key, separator, value = line.partition(" = ")
        if not separator or not value or any(char in value for char in "{}\r\n"):
            raise ValueError(f"{path}:{number}: expected a literal single-line Fluent message")
        if key in result:
            raise ValueError(f"{path}:{number}: duplicate Fluent message {key}")
        result[key] = value
    return result


def build_catalogs(data: Path, ftl: Path, policy: CurrentData, icu: dict) -> tuple[Catalog, ...]:
    if set(icu) != {spec.standard for spec in SPECS if spec.icu_names}:
        raise ValueError("ICU catalog ownership differs from the extractor")
    sources = {
        source: read_json(data / "iso" / f"iso_{source}.json")[source]
        for source in sorted({spec.source for spec in SPECS})
    }
    catalogs = []
    for spec in SPECS:
        records = sources[spec.source]
        if spec.source == "finite":
            records = records[spec.standard]
        records = sorted(
            (record for record in records if spec.code_key in record),
            key=lambda record: record[spec.code_key],
        )
        if spec.icu_names:
            if set(icu[spec.standard]) != {record[spec.code_key] for record in records}:
                raise ValueError(f"ICU snapshot codes differ from ISO source: {spec.standard}")
            records = [record for record in records
                       if icu[spec.standard][record[spec.code_key]] is not None]
        records = [record for record in records
                   if policy.includes(spec.standard, record, spec.code_key)]
        labels = {} if spec.icu_names else read_ftl(ftl / f"{spec.standard}.ftl")
        selected = []
        for record in records:
            code = record[spec.code_key]
            message_id = f"iso-{spec.standard}-{code}"
            name = (icu[spec.standard][code] if spec.icu_names
                    else labels.get(message_id, record["name"]))
            if not name or any(char in name for char in "{}\r\n"):
                raise ValueError(f"invalid English label: {message_id}")
            variant = record.get("variant") or "".join(
                part[0].upper() + part[1:].lower() for part in code.split("-")
            )
            aliases = tuple(record[key] for key in ("alpha_2", "alpha_3", "alpha_4", "numeric")
                            if key in record and record[key] != code)
            metadata = {key: record[key] for key in sorted(METADATA - {spec.code_key})
                        if key in record}
            selected.append(Entry(code, variant, name, message_id, aliases, metadata))
        if not selected:
            raise ValueError(f"empty selected catalog: {spec.standard}")
        for field in ("code", "variant"):
            if len({getattr(entry, field) for entry in selected}) != len(selected):
                raise ValueError(f"duplicate {field}: {spec.standard}")
        catalogs.append(Catalog(spec, tuple(selected)))
    return tuple(catalogs)
