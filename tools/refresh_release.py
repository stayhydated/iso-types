"""Repin one released upstream's data/current inputs, such as `cldr 49.0.0`.

Usage: python3 tools/refresh_release.py {cldr,tz,libphonenumber} VERSION

Resolves the release tag to its commit, downloads every input recorded for that
upstream in data/current/provenance.json from that commit, and updates URLs,
checksums, and the pin. Run `python3 -m tools.codegen` afterwards and review the
regenerated data.
"""

from datetime import date
import hashlib
import json
from pathlib import Path
import sys
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))

from tools.codegen import releases  # noqa: E402


def main(name: str, version: str) -> None:
    upstream = releases.BY_NAME[name]
    path = ROOT / "data/current/provenance.json"
    provenance = json.loads(path.read_text(encoding="utf-8"))
    tag = upstream.tag(version)
    revision = releases.resolve(upstream, tag)
    for file, source in provenance["files"].items():
        if not source["url"].startswith(upstream.raw):
            continue
        relative = source["url"][len(upstream.raw):].split("/", 1)[1]
        url = upstream.raw + revision + "/" + relative
        with urllib.request.urlopen(url) as response:
            content = response.read()
        (path.parent / file).write_bytes(content)
        source.update(url=url, sha256=hashlib.sha256(content).hexdigest())
        print(f"{file}: {relative}")
    provenance["releases"][name] = {
        "version": version,
        "tag": tag,
        "revision": revision,
        "retrieved": date.today().isoformat(),
    }
    releases.verify_pins(provenance)
    path.write_text(json.dumps(provenance, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(f"pinned {name} {version} ({tag} at {revision})")


if __name__ == "__main__":
    if len(sys.argv) != 3 or sys.argv[1] not in releases.BY_NAME:
        sys.exit(__doc__)
    main(*sys.argv[1:])
