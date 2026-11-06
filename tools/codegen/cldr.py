"""Versioned CLDR pin: a release tag, its resolved commit, and ICU4X agreement."""

import json
from pathlib import Path
import re
import subprocess

REPOSITORY = "https://github.com/unicode-org/cldr"
RAW = "https://raw.githubusercontent.com/unicode-org/cldr/"


def tag_for(version: str) -> str:
    """CLDR tags drop zero components: 49.0.0 is release-49, 48.2.0 is release-48-2."""
    match = re.fullmatch(r"(\d+)\.(\d+)\.(\d+)", version)
    if not match:
        raise ValueError(f"CLDR version must be MAJOR.MINOR.PATCH: {version}")
    parts = [match[1]] + ([match[2]] if match[2] != "0" or match[3] != "0" else []) \
        + ([match[3]] if match[3] != "0" else [])
    return "release-" + "-".join(parts)


def verify_pin(provenance: dict) -> dict:
    """Every CLDR input must come from the pinned release's resolved commit."""
    pin = provenance["cldr"]
    if pin["tag"] != tag_for(pin["version"]):
        raise ValueError(f"CLDR tag {pin['tag']} does not name version {pin['version']}")
    if not re.fullmatch(r"[0-9a-f]{40}", pin["revision"]):
        raise ValueError(f"CLDR revision must be a full commit hash: {pin['revision']}")
    prefix = RAW + pin["revision"] + "/"
    for name, source in provenance["files"].items():
        url = source["url"]
        if url.startswith(RAW) and not url.startswith(prefix):
            raise ValueError(f"{name} is not pinned to CLDR {pin['tag']} ({pin['revision']})")
    return pin


def icu_versions(root: Path) -> set[str]:
    """CLDR versions declared by the locked ICU4X compiled-data crates."""
    metadata = json.loads(subprocess.run(
        ["cargo", "metadata", "--format-version", "1", "--locked"],
        cwd=root, check=True, capture_output=True, text=True,
    ).stdout)
    return {
        package["metadata"]["sources"]["cldr"]["tagged"]
        for package in metadata["packages"]
        if package["name"].startswith("icu_") and package["name"].endswith("_data")
        and (package.get("metadata") or {}).get("sources", {}).get("cldr")
    }


def verify_icu_agreement(root: Path, pin: dict) -> None:
    """CLDR-derived tables and ICU4X-extracted names must describe one CLDR release."""
    versions = icu_versions(root)
    if versions != {pin["version"]}:
        found = ", ".join(sorted(versions)) or "none"
        raise ValueError(
            f"pinned CLDR {pin['version']} differs from ICU4X compiled data ({found}); "
            f"run python3 tools/refresh_cldr.py {', '.join(sorted(versions))}"
        )


def resolve(tag: str) -> str:
    """Resolve a release tag to its commit, peeling annotated tags."""
    output = subprocess.run(
        ["git", "ls-remote", "--tags", REPOSITORY, f"refs/tags/{tag}", f"refs/tags/{tag}^{{}}"],
        check=True, capture_output=True, text=True,
    ).stdout
    refs = dict(reversed(line.split("\t")) for line in output.splitlines())
    commit = refs.get(f"refs/tags/{tag}^{{}}") or refs.get(f"refs/tags/{tag}")
    if not commit:
        raise ValueError(f"CLDR tag not found: {tag}")
    return commit
