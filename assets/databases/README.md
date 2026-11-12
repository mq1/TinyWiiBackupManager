# Game databases

What the app identifies games with. Shipped with the app (built into it), so that it works
offline; refreshed with `python3 scripts/update-databases.py` from the repository root.

The app also refreshes them itself (`src/romdb/update.rs`): once a day it asks libretro-database
for the DATs that changed since it last asked (by ETag), compacts them the same way as the script
and keeps a list in `databases/no-intro/` in its data folder when it is newer (by the DAT's
version, a date) than the shipped one. When the app ships a newer list later, that one is used
again. A test checks that both give the same files:
`UGBM_DAT_DIR=<folder with the six DATs> cargo test -- --ignored compacts_the_real_dats`.

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
