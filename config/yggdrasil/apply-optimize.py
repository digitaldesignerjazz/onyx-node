#!/usr/bin/env python3
"""Patch /etc/yggdrasil/yggdrasil.conf with Onyx-safe defaults.

Touches only AdminListen, Listen, IfName, IfMTU, NodeInfoPrivacy,
NodeInfo, MulticastInterfaces. Leaves PrivateKey and Peers alone.

  sudo python3 config/yggdrasil/apply-optimize.py
  sudo systemctl restart yggdrasil
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

DST = Path("/etc/yggdrasil/yggdrasil.conf")

MULTICAST = """MulticastInterfaces: [
  {
    Regex: "eth.*"
    Beacon: true
    Listen: true
    Port: 0
    Priority: 0
  }
  {
    Regex: ".*"
    Beacon: false
    Listen: false
  }
]"""


def set_scalar(conf: str, key: str, value: str) -> str:
    pattern = re.compile(rf"^{re.escape(key)}:\s*.*$", re.MULTILINE)
    line = f"{key}: {value}"
    if pattern.search(conf):
        return pattern.sub(line, conf, count=1)
    return conf.rstrip() + "\n\n" + line + "\n"


def replace_block(conf: str, key: str, block: str) -> str:
    pattern = re.compile(rf"{re.escape(key)}:\s*\[[\s\S]*?\]", re.MULTILINE)
    if pattern.search(conf):
        return pattern.sub(block, conf, count=1)
    return conf.rstrip() + "\n\n" + block + "\n"


def replace_object(conf: str, key: str, block: str) -> str:
    pattern = re.compile(rf"{re.escape(key)}:\s*\{{[\s\S]*?\}}", re.MULTILINE)
    if pattern.search(conf):
        return pattern.sub(block, conf, count=1)
    return conf.rstrip() + "\n\n" + block + "\n"


def main() -> None:
    if not DST.is_file():
        raise SystemExit(f"missing {DST}")
    original = DST.read_text()
    if "PrivateKey:" not in original:
        raise SystemExit("refusing to write a config without PrivateKey")

    updated = original
    updated = set_scalar(updated, "AdminListen", "unix:///var/run/yggdrasil/yggdrasil.sock")
    updated = replace_block(updated, "Listen", "Listen: []")
    updated = set_scalar(updated, "IfName", "ygg0")
    updated = set_scalar(updated, "IfMTU", "65535")
    updated = set_scalar(updated, "NodeInfoPrivacy", "true")
    updated = replace_object(updated, "NodeInfo", "NodeInfo: {}")
    updated = replace_block(updated, "MulticastInterfaces", MULTICAST)

    if "PrivateKey:" not in updated:
        raise SystemExit("refusing to write a config without PrivateKey")
    if updated == original:
        print("already optimized")
        return

    backup = DST.with_suffix(".conf.opt.bak")
    backup.write_text(original)
    DST.write_text(updated)
    print(f"backup: {backup}")
    print(f"updated: {DST}")
    print("changed: AdminListen IfName IfMTU NodeInfoPrivacy MulticastInterfaces Listen")
    print("untouched: PrivateKey Peers")


if __name__ == "__main__":
    try:
        main()
    except PermissionError:
        sys.exit("permission denied — run with sudo")
