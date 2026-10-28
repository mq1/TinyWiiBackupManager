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

### M0 · Loose ends (S) – done

- Writing actions check first whether the drive is writable: one clear message ("WiiGames is
  read-only (NTFS, which macOS only reads)") instead of one per file; the drive card shows it.
- The same file in several sources is one entry with "2 copies".
- A drive's games and homebrew are tabs below a bar with the way back to My Consoles.
- The library shows the last scan at once and scans again in the background. (Ignoring folders
  turned out not to be needed: the media folders scan in under a second; the slow part was the
  cold hard disk.)

### M1a · Task manager with progress bars (M) – before M1

Every longer action – import, copy to a drive, install on a console, convert, export, scrub,
checksums, downloads, scans, later unpacking and injecting – becomes a **task** in one task manager.
It replaces TWBM's one slot per kind of action (`LongOperations`) and its separate import queue.

- **Progress bars** with bytes and percent, speed and time left, for every task; the 3DS shows its
  own bar during installs, as vcinject Link already does.
- **In parallel where it makes sense**: tasks run side by side, but each *resource* – a drive, a
  console's connection (vcinject Link serves one at a time), the network – has its own queue, so
  two copies to the same disk don't slow each other down, while a copy to WiiGames and an install
  on the 3DS run at the same time.
- **Control**: pause, cancel (cleaning up half-written files), retry a failed task, change the order
  of waiting tasks.
- **Where to see it**: a compact bar at the bottom of every page (number of running tasks, overall
  progress), opening the task manager page with the list: running, waiting, done, failed with
  the reason. Finished tasks stay in a history until cleared.
- **After a restart**: unfinished tasks are listed again and can be resumed or dropped.
- Built on iced's `Task::sipper`/streams for progress, with a cancel token per task; the existing
  import, convert, export, scrub and hash code reports progress through it.
- **The computer stays awake while data moves**: as long as a task copies, installs, imports or
  downloads, the app keeps the computer from going to sleep from idleness (macOS: an IOKit
  power assertion like `caffeinate -i`; Windows: `SetThreadExecutionState`; Linux:
  `systemd-inhibit`/the portal's Inhibit), and lets go when the last such task ends. The screen may
  still turn off. Found while 116 Wii games were imported to the WiiGames drive.

### M1a+ · Notifications and log (S–M)

TWBM's notifications are toasts in the window that pile up (one per error, e.g. hundreds of
"Read-only file system"). Rework them together with the task manager:

- **The system's notifications** for what matters while the app is in the background: a task
  finished or failed, a console came online, an update is ready (macOS: User Notifications;
  Windows: toast notifications; Linux: the freedesktop notification spec). Can be turned off.
- **An own notification center** in the app: all messages in one list, grouped (many errors of one
  task are one entry with "and 245 more"), with time, source (task, console, drive) and a link to
  what they are about.
- **The log in the same place**: choose a level – errors, warnings, info, debug – and the center
  shows the log lines of that level next to the notifications, searchable and exportable as a file
  for bug reports. The log is also written to a file in the data folder.


### M1 · Install on the 3DS over Wi-Fi (M)

- "Install" on a CIA in the library: choose the 3DS; it runs as a task (M1a), with progress on the
  computer and on the 3DS.
- Delete and launch titles from the console's title list.
- Offer the update of vcinject Link when it is older than the app expects.

### M1b · Prepare SD cards and drives (M)

A wizard in My Consoles ("Prepare an SD card or drive") that makes a card or disk ready for a
console: check it, format it if needed – with the tools the operating system brings – and lay out
what the console expects.

**1. Choose the target and the disk**
- Target: Wii SD card, Wii/GameCube USB drive (USB loaders, Nintendont), Wii U SD card, 3DS SD
  card. Wii U game drives are formatted by the Wii U itself (WFS); the wizard only says so.
- Only external and removable disks are offered, never the system disk. Each shows model, size,
  partition scheme, file system and cluster size.

**2. Check what is there**
- Wanted: MBR (not GPT) with one primary FAT32 partition; clusters of 32 KiB up to 32 GiB, 64 KiB
  above (the rule TWBM's drive info already uses). The 3DS wants FAT32 with 64 KiB clusters too.
- If it already fits: no formatting, go on with the layout. If not: say exactly what is wrong
  (e.g. "NTFS: the Wii can't use it, and macOS can only read it").

**3. Back up, then format (only when needed)**
- Before formatting, offer to copy the contents into a ROM source (games, saves, `apps/`) and show
  how much that is.
- Formatting erases everything: the disk's name and size are shown again and must be confirmed by
  typing its name.
- The operating system's own tools do the work; the app builds the command, shows it and runs it
  with the rights it needs:

| OS | Partition + format | Rights |
| --- | --- | --- |
| macOS | `diskutil eraseDisk FAT32 <NAME> MBRFormat /dev/diskN`, then `newfs_msdos -F 32 -c <sectors per cluster> -v <NAME>` on the partition for the cluster size | `diskutil` none; `newfs_msdos` needs administrator rights (password prompt), on the unmounted raw partition `/dev/rdiskNsM`; `-c 128` = 64 KiB clusters (`diskutil` alone picks 32 KiB) |
| Windows | `diskpart` script (`clean`, `convert mbr`, `create partition primary`, `format fs=fat32 unit=<size> quick label=<NAME>`, `assign`) | administrator (UAC prompt) |
| Linux | `parted` (`mklabel msdos`, `mkpart primary fat32`) and `mkfs.fat -F 32 -s <sectors per cluster> -n <NAME>` | `pkexec` |

- Windows refuses FAT32 above 32 GB with its own tools in older versions; newer Windows 11
  versions are said to lift that limit for the command line. To verify; where it doesn't work, the
  wizard explains it instead of falling back to a third-party formatter.

**4. Lay out the card or drive**
- Folders per target: `wbfs/` and `games/` (Wii/GameCube drive), `apps/` (Wii SD), `wiiu/` (Wii U
  SD, Aroma), `3ds/`, `luma/`, `cias/` (3DS SD).
- Optionally the usual homebrew, downloaded at that moment from its official source (Open Shop
  Channel, Universal-DB, the projects' GitHub releases): e.g. USB Loader GX and Nintendont, Aroma,
  Luma3DS's `boot.firm`. Nothing that isn't freely distributable is bundled.
- Then the card or drive is remembered as a device in My Consoles, and the backup (if one was
  made) can be copied back.

**5. Safety**
- The command is shown before it runs, and its output is kept in the app's log.
- A disk that changed between choosing and formatting (unplugged, other disk at the same name) stops
  the wizard.
- Tested first with disk images (`hdiutil` on macOS, loop devices on Linux, VHDs on Windows) in CI,
  never with a real disk.

### M1c · Copy many games to a drive (S)

- "Copy to drive" in the library for a selection, a platform or a whole source folder, not one game
  at a time; files over 4 GiB are always split for FAT32 (`.wbfs` + `.wbf1` …), as the drive import
  already does. Found when copying the WBFS backup to the freshly formatted WiiGames drive, where
  the Finder refuses files over 4 GiB.

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
- Name matching alone isn't good enough for series ("The Legend of Zelda", "Zelda II" and "Link's
  Awakening" are one series; "Metroid" and "Super Metroid" too). So: keep the name heuristic, and
  add a hand-made list of important series (Mario, Zelda, Pokémon, Metroid, Kirby, Donkey Kong …)
  with their games' IDs, which wins over the heuristic.
- Research before building: better sources for series (e.g. Wikidata's "part of the series",
  IGDB's franchises, MobyGames groups – licenses and offline use to check).

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
- Whether a source for series exists that is good, open and usable offline (see M5).
- Formatting FAT32 above 32 GB with Windows' own tools: which versions allow it (see M1b).
