# Roadmap

The master plan for Ultimate Game Backup Manager, as of 2026-10-28. It replaces the phase list
in the README where the two differ.

## Where we are

Done and tried on real hardware (the 3DS "petter1", the iCloud collection and the gamestation drive):

- **Fork**: renamed, credits, AI policy, CI on Linux, macOS and Windows.
- **[Link protocol](LINK_PROTOCOL.md)** for all companion apps, and its Rust client.
- **My Consoles**: finds 3DS consoles running vcinject Link and shows their titles with icons;
  Wii/GameCube drives are devices there too (added once, remembered).
- **Library** (the Games page): the collection from several ROM sources, filtered by platform.
  CIAs, discs, WADs, Wii U titles, ROMs, RAR releases and archives, SD card copies; files only in
  iCloud aren't downloaded. Shows which online console a title is installed on.

## How the app is meant to work

- **Library and devices are separate.** ROM sources are folders on the computer; devices are
  consoles (over the network) and drives. The library shows what you have, devices show what is on
  them.
- **One entry per game, on its original platform.** The ROM or disc and all injects made from it
  (3DS CIA, Wii U inject) are inside that entry. A game shows the consoles it can be installed on;
  the library can be filtered by that target console.
- **Games are identified, not guessed**: by the checksum of the ROM or disc (also the one inside
  an inject) against imported databases. We don't build our own database.
- **Missing games** are the games in the databases that aren't in the collection.
- **Collections** (series or sets, e.g. all Zelda games) are a database of their own with
  references to games. A game can be in several collections, and a collection can contain games
  you don't have yet.

## Milestones

Each milestone ends in something usable. They are in the planned order; sizes are rough
(S = a few hours, M = about one working session like 2026-10-28, L = several).

### M0 · Loose ends (S)

- Writing tools check first whether a drive is writable: one clear message ("WiiGames is
  read-only: NTFS on macOS") instead of one per file; the drive card shows it.
- The same file in two sources is one entry with "2 copies".
- A drive's games and homebrew open as tabs in its card in My Consoles.
- Ignore folders in a source (e.g. `video/`, `music/` on gamestation) to keep scans fast.

### M1 · Install on the 3DS over Wi-Fi (M)

- "Install" on a CIA in the library: choose the 3DS, progress on the computer and on the 3DS.
- Delete and launch titles from the console's title list.
- Offer the update of vcinject Link when it is older than the app expects.

### M2 · Game databases (M)

- Import and keep up to date: libretro-database (No-Intro, Redump), GameTDB (Wii, GameCube,
  Wii U, 3DS), 3dsdb and WiiUBrew title lists – only the platforms we support, compacted.
- **The app ships the databases and updates them.** It keeps working when a source is down.
  - Every build pulls the current databases.
  - A snapshot lives in the repository, so that a `git clone` brings current databases along and
    a build works offline; the build uses it when it can't download, and refreshes it when it can.
  - At run time the app checks for newer databases and falls back to the shipped ones.
- Identify every library entry: by checksum (for RAR releases from the RAR headers), by title or
  game ID, by name as a last resort.
- RAR is read and unpacked with an external **7-Zip** (`7z`, `7zz` on macOS; on every platform,
  handles RAR5), falling back to `unar`/`lsar` when only that is installed. The unrar library's
  license doesn't fit GPL, so it isn't linked.
- Real names for the ~700 entries that are only a title ID today.

### M3 · Injects by content (M–L)

- Find the ROM or disc inside an inject and identify it by its checksum:
  - 3DS GBA VC: ExeFS `.code` = ROM + footer with the ROM size.
  - 3DS forwarders (SNES, NES, GB …): the ROM in the RomFS.
  - Wii U, unpacked: `.z64` (N64), `rom.zip` (DS), ISO (GameCube), `.nfs` (Wii).
  - Wii U as WUP (encrypted): needs the Wii U common key, entered once by the user; never shipped.
  - Wii U GBA (`alldata.psb.m`) and NES/SNES (inside the `.rpx`): later.
- Extract the ROM out of an inject again.
- RAR releases are unpacked for this in the background, once, and the result is cached.

### M4 · One entry per game (M)

- The library groups files and injects under their game.
- Game view: files, injects, where it is installed, "installable on" (GBA → 3DS, Wii U; GameCube
  → Wii with Nintendont, Wii U; Wii → Wii, Wii U; N64, DS, SNES … → Wii U).
- Filter by target console.

### M5 · Missing games and collections (M)

- Per platform: have / missing, from the databases.
- Collections: their own file in the data folder with references to game IDs; create, edit, add
  games; progress per collection ("Zelda: 14 of 19").
- Series are suggested from the database names, as vcinject's `library.series` does ("Pokemon -
  Emerald Version" and "Pokemon Mystery Dungeon" → "Pokemon", with exceptions like "Mario Andretti
  Racing"); "missing" leaves out betas, hacks, BIOS and the like, as vcinject's catalog does.
  Port both.

### M6 · The rest of the 3DS (M–L)

- Saves: back up and restore.
- HOME Menu: folders and order.
- Universal-DB: homebrew store.
- Injector: build 3DS injects from the library, first through vcinject as a command-line sidecar,
  later ported to Rust.

### M7 · Wii (L)

- Companion app (libogc) speaking the Link protocol: title list, WAD install, saves.
- Drives: FAT32 guidance and formatting help (the WiiGames drive is being reformatted).

### M8 · Wii U (L)

- Companion app for Aroma (`.wuhb`) with WUP installs like WUP Installer GX2, title list, saves.
- Injector modelled on UWUVCI (GPL-3.0, so compatible).

## Open questions

- How large the database snapshot gets, and whether it needs Git LFS.
- How collections are shared between people.
