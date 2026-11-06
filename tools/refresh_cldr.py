"""Repin data/current CLDR inputs to a CLDR release, such as `49.0.0`.

Usage: python3 tools/refresh_cldr.py VERSION

Resolves the release tag to its commit, downloads every CLDR input recorded in
data/current/provenance.json from that commit, and updates URLs, checksums, and
the pin. Run `python3 -m tools.codegen` afterwards and review the regenerated data.
"""

from datetime import date
import hashlib
import json
from pathlib import Path
import sys
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))

from tools.codegen import cldr  # noqa: E402


def main(version: str) -> None:
    path = ROOT / "data/current/provenance.json"
    provenance = json.loads(path.read_text(encoding="utf-8"))
    tag = cldr.tag_for(version)
    revision = cldr.resolve(tag)
    for name, source in provenance["files"].items():
        if not source["url"].startswith(cldr.RAW):
            continue
        relative = source["url"][len(cldr.RAW):].split("/", 1)[1]
        url = cldr.RAW + revision + "/" + relative
        with urllib.request.urlopen(url) as response:
            content = response.read()
        (path.parent / name).write_bytes(content)
        source.update(url=url, sha256=hashlib.sha256(content).hexdigest())
        print(f"{name}: {relative}")
    provenance["cldr"] = {
        "version": version,
        "tag": tag,
        "revision": revision,
        "retrieved": date.today().isoformat(),
    }
    cldr.verify_pin(provenance)
    path.write_text(json.dumps(provenance, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(f"pinned CLDR {version} ({tag} at {revision})")


if __name__ == "__main__":
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    main(sys.argv[1])
