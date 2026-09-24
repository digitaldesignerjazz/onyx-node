#!/usr/bin/env python3
"""Replace the Peers block in the local Yggdrasil config.

Reads peer URIs from peers-de.conf next to this script.
Rewrites only the Peers list in /etc/yggdrasil/yggdrasil.conf.
Leaves PrivateKey and every other field untouched.

Run as root:
  sudo python3 config/yggdrasil/apply-peers.py
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

SRC = Path(__file__).with_name("peers-de.conf")
DST = Path("/etc/yggdrasil/yggdrasil.conf")

def peer_uris(text: str) -> list[str]:
    uris = []
    for line in text.splitlines():
        line = line.split("#", 1)[0].strip().rstrip(",")
        if "://" in line:
            uris.append(line.strip().strip('"').strip("'"))
    if not uris:
        raise SystemExit(f"no peer URIs in {SRC}")
    return uris

def replace_peers(conf: str, uris: list[str]) -> str:
    block = "Peers: [\n" + "".join(f"    {u}\n" for u in uris) + "  ]"
    pattern = re.compile(r"Peers:\s*\[[\s\S]*?\]", re.MULTILINE)
    match = pattern.search(conf)
    if not match:
        raise SystemExit("Peers block not found in yggdrasil.conf")
    return conf[: match.start()] + block + conf[match.end() :]

def main() -> None:
    if not SRC.is_file():
        raise SystemExit(f"missing {SRC}")
    if not DST.is_file():
        raise SystemExit(f"missing {DST}")
    uris = peer_uris(SRC.read_text())
    original = DST.read_text()
    updated = replace_peers(original, uris)
    if "PrivateKey:" not in updated:
        raise SystemExit("refusing to write a config without PrivateKey")
    if updated == original:
        print("Peers already set:")
        for u in uris:
            print(f"  {u}")
        return
    backup = DST.with_suffix(".conf.bak")
    backup.write_text(original)
    DST.write_text(updated)
    print(f"backup: {backup}")
    print(f"updated: {DST}")
    for u in uris:
        print(f"  {u}")

if __name__ == "__main__":
    try:
        main()
    except PermissionError:
        sys.exit("permission denied — run with sudo")
