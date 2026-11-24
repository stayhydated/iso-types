"""Check that a locale-only library tracks configuration changes through compiler caches."""

import argparse
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parent.parent
HASH_ENV = "ES_FLUENT_BUILD_MANIFEST_HASH"
CARGO = os.environ.get("CARGO", "cargo")


def fluent_packages():
    metadata = json.loads(subprocess.check_output(
        [CARGO, "metadata", "--format-version=1", "--locked"], cwd=ROOT, text=True,
    ))
    packages = {}
    sources = set()
    for package in metadata["packages"]:
        name = package["name"]
        if not name.startswith("es-fluent"):
            continue
        if name in packages:
            raise RuntimeError(f"multiple {name} packages in the integration dependency graph")
        packages[name] = str(Path(package["manifest_path"]).parent)
        source = package["source"] or ""
        if source.startswith("git+"):
            sources.add(source.removeprefix("git+").split("?")[0].split("#")[0])
    return packages, sources


def write_fixture(directory, packages, sources):
    # Root patch tables do not propagate into a separate consumer workspace.
    # Reuse the exact resolved sources, including the unpublished manager API.
    patches = "".join(
        f"\n[patch.{json.dumps(source)}]\n" + "".join(
            f"{name} = {{ path = {json.dumps(path)} }}\n"
            for name, path in sorted(packages.items())
        )
        for source in ["crates-io", *sorted(sources)]
    )
    (directory / "Cargo.toml").write_text(f"""[package]
name = "iso-types-locale-tracking"
version = "0.0.0"
edition = "2024"
publish = false

[workspace]

[dependencies]
iso-types = {{ path = {json.dumps(str(ROOT / 'crates/iso-types'))}, default-features = false, features = ["locales"] }}

[build-dependencies]
es-fluent-build = {{ path = {json.dumps(packages['es-fluent-build'])} }}
{patches}""")
    shutil.copyfile(ROOT / "Cargo.lock", directory / "Cargo.lock")
    (directory / "build.rs").write_text(
        "fn main() { es_fluent_build::track_i18n_assets(); }\n"
    )
    (directory / "src").mkdir()
    (directory / "src/lib.rs").write_text("""use iso_types::VariantArray as _;

#[iso_types::iso_locale]
pub enum Languages {}

pub fn snapshot() -> String {
    let locales = Languages::VARIANTS
        .iter()
        .map(|locale| locale.code())
        .collect::<Vec<_>>()
        .join(",");
    format!("{locales};{}", Languages::default().code())
}
""")
    (directory / "src/main.rs").write_text(
        'fn main() { println!("{}", iso_types_locale_tracking::snapshot()); }\n'
    )
    for locale in ("en", "fr-FR"):
        (directory / "i18n" / locale).mkdir(parents=True)
    set_fallback(directory, "en")


def set_fallback(directory, locale):
    (directory / "i18n.toml").write_text(
        f'fallback_language = "{locale}"\nassets_dir = "i18n"\n'
    )


def check_snapshot(directory, env, expected, *, locked=True):
    # The first build adjusts the copied workspace lock for this consumer. All
    # dependencies are already resolved by metadata; no network updates occur.
    args = [CARGO, "run", "--quiet", "--offline"]
    if locked:
        args.append("--locked")
    result = subprocess.run(args, cwd=directory, env=env, text=True, stdout=subprocess.PIPE)
    if result.returncode:
        print(result.stdout, file=sys.stderr, end="")
        result.check_returncode()
    actual = result.stdout.strip()
    if actual != expected:
        raise AssertionError(f"stale locale enum: expected {expected!r}, got {actual!r}")
    debug = directory / "target/debug"
    # Cargo 1.100 stores intermediate dep-info in the per-package build
    # directory. Keep the previous layout for running the fixture on 1.99.
    dep_files = [
        path
        for pattern in (
            "deps/iso_types_locale_tracking-*.d",
            "build/iso-types-locale-tracking/*/out/iso_types_locale_tracking-*.d",
        )
        for path in debug.glob(pattern)
        if "src/lib.rs" in path.read_text()
    ]
    if not dep_files:
        raise AssertionError("the locale-only library did not emit dep-info")
    dep_info = max(dep_files, key=lambda path: path.stat().st_mtime_ns).read_text()
    hashes = re.findall(rf"^# env-dep:{HASH_ENV}=(.+)$", dep_info, flags=re.MULTILINE)
    if len(hashes) != 1:
        raise AssertionError("rustc did not record the locale manifest hash in dep-info")
    print(f"validated {actual}", flush=True)
    return hashes[0]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--sccache", action="store_true", help="require and use sccache")
    args = parser.parse_args()
    env = os.environ.copy()
    if args.sccache:
        wrapper = shutil.which("sccache")
        if wrapper is None:
            parser.error("--sccache requires sccache on PATH")
        env["RUSTC_WRAPPER"] = wrapper
    # sccache cannot cache incrementally compiled Rust crates.
    env["CARGO_INCREMENTAL"] = "0"
    packages, sources = fluent_packages()
    (ROOT / "target").mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="locale-tracking-", dir=ROOT / "target") as temporary:
        directory = Path(temporary)
        env["CARGO_TARGET_DIR"] = str(directory / "target")
        write_fixture(directory, packages, sources)
        previous = check_snapshot(directory, env, "en,fr-FR;en", locked=False)

        set_fallback(directory, "fr-FR")
        current = check_snapshot(directory, env, "en,fr-FR;fr-FR")
        if current == previous:
            raise AssertionError("changing the fallback did not change the tracked hash")
        previous = current

        (directory / "i18n/de-DE").mkdir()
        current = check_snapshot(directory, env, "de-DE,en,fr-FR;fr-FR")
        if current == previous:
            raise AssertionError("adding a locale did not change the tracked hash")
        previous = current

        (directory / "i18n/de-DE").rename(directory / "i18n/ja")
        current = check_snapshot(directory, env, "en,fr-FR,ja;fr-FR")
        if current == previous:
            raise AssertionError("renaming a locale did not change the tracked hash")
        previous = current

        (directory / "i18n/ja").rmdir()
        current = check_snapshot(directory, env, "en,fr-FR;fr-FR")
        if current == previous:
            raise AssertionError("removing a locale did not change the tracked hash")


if __name__ == "__main__":
    main()
