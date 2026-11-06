"""Versioned upstream pins: a release tag, its resolved commit, and the Rust crate that bundles it.

CLDR, tz, and libphonenumber inputs come from tagged releases. Each must equal the
release compiled into the locked runtime dependency, so generated catalogs and
runtime behavior describe one dataset.
"""

from collections.abc import Callable
from dataclasses import dataclass
import json
from pathlib import Path
import re
import subprocess


def cldr_tag(version: str) -> str:
    """CLDR tags drop zero components: 49.0.0 is release-49, 48.2.0 is release-48-2."""
    match = re.fullmatch(r"(\d+)\.(\d+)\.(\d+)", version)
    if not match:
        raise ValueError(f"CLDR version must be MAJOR.MINOR.PATCH: {version}")
    parts = [match[1]] + ([match[2]] if match[2] != "0" or match[3] != "0" else []) \
        + ([match[3]] if match[3] != "0" else [])
    return "release-" + "-".join(parts)


def tz_tag(version: str) -> str:
    if not re.fullmatch(r"\d{4}[a-z]", version):
        raise ValueError(f"tz version must look like 2026e: {version}")
    return version


def libphonenumber_tag(version: str) -> str:
    if not re.fullmatch(r"\d+\.\d+\.\d+", version):
        raise ValueError(f"libphonenumber version must be MAJOR.MINOR.PATCH: {version}")
    return "v" + version


def icu_cldr(packages: list[dict]) -> set[str]:
    """ICU4X compiled-data crates declare their CLDR release in package metadata."""
    return {
        package["metadata"]["sources"]["cldr"]["tagged"]
        for package in packages
        if package["name"].startswith("icu_") and package["name"].endswith("_data")
        and ((package.get("metadata") or {}).get("sources") or {}).get("cldr")
    }


def jiff_tz(packages: list[dict]) -> set[str]:
    """jiff-tzdb records its bundled tz release as `VERSION` in `tzname.rs`."""
    versions = set()
    for package in packages:
        if package["name"] != "jiff-tzdb":
            continue
        source = (Path(package["manifest_path"]).parent / "tzname.rs").read_text(encoding="utf-8")
        match = re.search(r'VERSION: Option<&str> = Some\(r?"([^"]+)"\)', source)
        if not match:
            raise ValueError(f"jiff-tzdb {package['version']} does not declare its tz release")
        versions.add(match[1])
    return versions


def phonenumber_metadata(packages: list[dict]) -> set[str]:
    """The phonenumber crate names its libphonenumber release as build metadata: 0.3.10+9.0.33."""
    return {
        package["version"].split("+", 1)[1]
        for package in packages
        if package["name"] == "phonenumber" and "+" in package["version"]
    }


@dataclass(frozen=True)
class Upstream:
    name: str
    repository: str
    tag: Callable[[str], str]
    bundled: Callable[[list[dict]], set[str]]
    bundled_by: str

    @property
    def raw(self) -> str:
        return f"https://raw.githubusercontent.com/{self.repository}/"


UPSTREAMS = (
    Upstream("cldr", "unicode-org/cldr", cldr_tag, icu_cldr, "ICU4X compiled data"),
    Upstream("tz", "eggert/tz", tz_tag, jiff_tz, "jiff-tzdb"),
    Upstream("libphonenumber", "google/libphonenumber", libphonenumber_tag,
             phonenumber_metadata, "the phonenumber crate"),
)
BY_NAME = {upstream.name: upstream for upstream in UPSTREAMS}


def verify_pins(provenance: dict) -> dict[str, dict]:
    """Every input from a released upstream must come from its pinned tag's commit."""
    pins = provenance["releases"]
    if set(pins) != set(BY_NAME):
        raise ValueError(f"release pins must cover exactly: {', '.join(sorted(BY_NAME))}")
    for upstream in UPSTREAMS:
        pin = pins[upstream.name]
        if pin["tag"] != upstream.tag(pin["version"]):
            raise ValueError(f"{upstream.name} tag {pin['tag']} does not name version {pin['version']}")
        if not re.fullmatch(r"[0-9a-f]{40}", pin["revision"]):
            raise ValueError(f"{upstream.name} revision must be a full commit hash: {pin['revision']}")
        prefix = upstream.raw + pin["revision"] + "/"
        for name, source in provenance["files"].items():
            url = source["url"]
            if url.startswith(upstream.raw) and not url.startswith(prefix):
                raise ValueError(f"{name} is not pinned to {upstream.name} {pin['tag']} ({pin['revision']})")
    return pins


def locked_packages(root: Path) -> list[dict]:
    return json.loads(subprocess.run(
        ["cargo", "metadata", "--format-version", "1", "--locked", "--all-features"],
        cwd=root, check=True, capture_output=True, text=True,
    ).stdout)["packages"]


def verify_agreement(pins: dict[str, dict], packages: list[dict]) -> None:
    """Pinned inputs and the releases bundled by locked Rust crates must be identical."""
    for upstream in UPSTREAMS:
        version = pins[upstream.name]["version"]
        versions = upstream.bundled(packages)
        if versions != {version}:
            found = ", ".join(sorted(versions)) or "none"
            raise ValueError(
                f"pinned {upstream.name} {version} differs from {upstream.bundled_by} ({found}); "
                f"run python3 tools/refresh_release.py {upstream.name} {found}"
            )


def resolve(upstream: Upstream, tag: str) -> str:
    """Resolve a release tag to its commit, peeling annotated tags."""
    output = subprocess.run(
        ["git", "ls-remote", "--tags", f"https://github.com/{upstream.repository}",
         f"refs/tags/{tag}", f"refs/tags/{tag}^{{}}"],
        check=True, capture_output=True, text=True,
    ).stdout
    refs = dict(reversed(line.split("\t")) for line in output.splitlines())
    commit = refs.get(f"refs/tags/{tag}^{{}}") or refs.get(f"refs/tags/{tag}")
    if not commit:
        raise ValueError(f"{upstream.name} tag not found: {tag}")
    return commit
