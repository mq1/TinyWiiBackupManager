# Game databases

What the app identifies games with. Shipped with the app (built into it), so that it works
offline; refreshed with `python3 scripts/update-databases.py` from the repository root.

## no-intro/

The [No-Intro](https://no-intro.org) lists of the cartridge consoles, as
[libretro-database](https://github.com/libretro/libretro-database) (`metadat/no-intro/`) has
them, compacted to what the app needs: one line per dump with

    CRC32 <tab> SHA-1 <tab> cartridge serial <tab> release name

The first line names the list and its version. The NES list has every dump twice in
libretro-database, with and without the iNES header; only the one without is kept, and the app
leaves the header out of a ROM's checksums.

| File | List |
| --- | --- |
| `gb.tsv` | Nintendo - Game Boy |
| `gbc.tsv` | Nintendo - Game Boy Color |
| `gba.tsv` | Nintendo - Game Boy Advance |
| `nes.tsv` | Nintendo - Nintendo Entertainment System |
| `snes.tsv` | Nintendo - Super Nintendo Entertainment System |
| `md.tsv` | Sega - Mega Drive - Genesis |

## License

libretro-database is licensed under
[CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/); the files in `no-intro/` are an
adaptation of it (compacted, nothing added) under the same license. Data: No-Intro, via
libretro-database and its contributors.
