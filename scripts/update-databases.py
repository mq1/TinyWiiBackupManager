#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Yanik Fuchs <yanik.fuchs@me.com>
# SPDX-License-Identifier: GPL-3.0-only
"""Refresh the game database snapshot in assets/databases/ from libretro-database.

Downloads the No-Intro DATs of the systems below from libretro-database
(https://github.com/libretro/libretro-database, CC BY-SA 4.0) and keeps only
what the app needs to identify a ROM, one line per dump:

    CRC32 <tab> SHA-1 <tab> serial <tab> release name

The result is a few hundred KB per system instead of the DAT's megabytes, and
plain text, so an update shows in a diff. Run it from the repository root:

    python3 scripts/update-databases.py
"""
from __future__ import annotations

import re
import sys
import urllib.parse
import urllib.request
from pathlib import Path

BASE = "https://raw.githubusercontent.com/libretro/libretro-database/master/metadat/no-intro/"
OUT = Path("assets/databases/no-intro")

# file name in the app -> DAT name in libretro-database
SYSTEMS = {
    "gb": "Nintendo - Game Boy",
    "gbc": "Nintendo - Game Boy Color",
    "gba": "Nintendo - Game Boy Advance",
    "nes": "Nintendo - Nintendo Entertainment System",
    "snes": "Nintendo - Super Nintendo Entertainment System",
    "md": "Sega - Mega Drive - Genesis",
}

_GAME = re.compile(r"^game \(\n(?P<body>.*?)^\)", re.S | re.M)
_NAME = re.compile(r'^\tname "(?P<name>(?:[^"\\]|\\.)*)"', re.M)
_ROM = re.compile(r'^\trom \( name "(?P<file>(?:[^"\\]|\\.)*)" (?P<fields>.*) \)$', re.M)
_FIELD = re.compile(r'(\w+) ("(?:[^"\\]|\\.)*"|\S+)')
_VERSION = re.compile(r'^\tversion "([^"]*)"', re.M)


def compact(dat: str, system: str) -> tuple[str, list[str]]:
    version = (_VERSION.search(dat) or [None, "unknown"])[1]
    lines = set()
    for game in _GAME.finditer(dat):
        body = game.group("body")
        name = _NAME.search(body).group("name")
        for rom in _ROM.finditer(body):
            # the NES list has every dump twice: with the iNES header (.nes) and without (.unh);
            # the app strips the header and looks up the headerless one
            if system == "nes" and not rom.group("file").endswith(".unh"):
                continue
            fields = {k: v.strip('"') for k, v in _FIELD.findall(rom.group("fields"))}
            if "crc" not in fields or "sha1" not in fields:
                continue
            serial = fields.get("serial", "")
            for text in (name, serial):
                if "\t" in text or "\n" in text:
                    raise ValueError(f"unexpected character in {text!r}")
            lines.add(f"{fields['crc'].upper()}\t{fields['sha1'].lower()}\t{serial}\t{name}")
    return version, sorted(lines, key=lambda line: (line.split("\t")[3].lower(), line))


def main() -> int:
    OUT.mkdir(parents=True, exist_ok=True)
    for key, dat_name in SYSTEMS.items():
        url = BASE + urllib.parse.quote(dat_name) + ".dat"
        with urllib.request.urlopen(url, timeout=60) as response:
            dat = response.read().decode("utf-8", "replace")
        version, lines = compact(dat, key)
        header = f"# {dat_name} (No-Intro via libretro-database, {version}); CC BY-SA 4.0\n"
        (OUT / f"{key}.tsv").write_text(header + "\n".join(lines) + "\n", encoding="utf-8")
        print(f"{key}: {len(lines)} dumps, version {version}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
