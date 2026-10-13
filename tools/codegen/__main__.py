"""Generate typed catalogs from pinned ISO/CLDR inputs, ICU4X names, and English FTL."""

import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys

from .catalog import SPECS, build_catalogs, verify_sources
from .output import ROOT, OutputPlan
from .rust import format_rust
from .policy import CurrentData
from . import render

DATA = ROOT / "data"
TYPES = ROOT / "crates/iso-types"
MACROS = ROOT / "crates/iso-types-macros"
FTL = TYPES / "i18n/en/iso-types"


def import_iso(checkout: Path) -> None:
    sources = sorted({spec.source for spec in SPECS} - {"finite"})
    plan = OutputPlan()
    hashes = {}
    for source in sources:
        name = f"iso_{source}.json"
        content = (checkout / "data" / name).read_bytes()
        json.loads(content)
        plan.add(DATA / "iso" / name, content)
        hashes[name] = hashlib.sha256(content).hexdigest()
    revision = subprocess.check_output(
        ["git", "-C", str(checkout), "rev-parse", "HEAD"], text=True,
    ).strip()
    provenance = {
        "source": "https://salsa.debian.org/iso-codes-team/iso-codes",
        "revision": revision,
        "sha256": hashes,
    }
    plan.add(DATA / "iso/provenance.json", json.dumps(provenance, indent=2) + "\n")
    plan.apply(check=False)


def generate(*, check: bool) -> None:
    verify_sources(DATA)
    policy = CurrentData(DATA)
    command = ["cargo", "run", "--quiet", "-p", "iso-types-icu-data", "--locked", "--", "--stdout"]
    extracted = subprocess.run(command, cwd=ROOT, check=True, capture_output=True, text=True)
    catalogs = build_catalogs(DATA, FTL, policy, json.loads(extracted.stdout)["catalogs"])
    plan = OutputPlan()
    plan.add(DATA / "icu/english.json", extracted.stdout)
    plan.add(TYPES / "src/catalogs.rs", format_rust(render.catalog_modules(catalogs)))
    for catalog in catalogs:
        plan.add(TYPES / "src/catalogs" / f"{catalog.spec.module}.rs", format_rust(render.catalog_rust(catalog)))
        if not catalog.spec.icu_names:
            plan.add(FTL / f"{catalog.spec.standard}.ftl", render.fluent_resource(catalog))
    plan.add(MACROS / "src/catalog/data.json", render.macro_data(catalogs))
    plan.add(TYPES / "src/i18n/resources.rs", format_rust(render.fluent_registration(catalogs)))
    scope = format_rust(policy.rust_locale_policy())
    for crate in (TYPES, MACROS):
        plan.add(crate / "src/locale/scope.rs", scope)
        for name in ("NOTICE", "LICENSE-APACHE", "LICENSE-MIT", "LICENSE-LGPL-2.1", "LICENSE-UNICODE"):
            plan.add(crate / name, (ROOT / name).read_text(encoding="utf-8"))
    changed = plan.apply(check=check)
    if check and changed:
        paths = "\n".join(f"  {path.relative_to(ROOT)}" for path in changed)
        raise ValueError(f"generated output differs:\n{paths}\nRun python3 -m tools.codegen to regenerate.")
    count = sum(len(catalog.entries) for catalog in catalogs)
    print(f"{len(catalogs)} catalogs, {count} current codes verified; {len(changed)} files updated")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--check", action="store_true", help="verify all generated outputs without writing")
    mode.add_argument("--import-data", type=Path, metavar="CHECKOUT", help="import an iso-codes checkout before generation")
    args = parser.parse_args()
    try:
        if args.import_data is not None:
            import_iso(args.import_data)
        generate(check=args.check)
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f"codegen: {error}", file=sys.stderr)
        if isinstance(error, subprocess.CalledProcessError) and error.stderr:
            print(error.stderr, file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
