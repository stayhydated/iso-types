"""Check isolated regional features and optionally the supported WASM target."""

import argparse
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parent.parent
FEATURES = (
    "regions", "regional-preferences", "unit-preferences", "language-relations",
    "time-zones", "time-zone-localization", "calling-codes", "phone-numbers", "addresses",
)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--wasm", action="store_true")
    args = parser.parse_args()
    for feature in FEATURES:
        subprocess.run(["cargo", "test", "-p", "iso-types", "--no-default-features",
                        "--features", feature, "--locked"], cwd=ROOT, check=True)
    if args.wasm:
        for package in ("iso-types", "iso-types-locale-consumer"):
            flags = ["--all-features"] if package == "iso-types" else []
            subprocess.run(["cargo", "check", "-p", package, *flags,
                            "--target", "wasm32-unknown-unknown", "--locked"], cwd=ROOT, check=True)


if __name__ == "__main__":
    main()
