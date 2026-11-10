# Roadmap

The master plan for Ultimate Game Backup Manager, as of 2026-11-10. It replaces the phase list
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
- **[Test log](TESTLOG.md)**: every test on real hardware is written down there – device, what,
  who, when, how, why and the result. From now on each milestone counts as done only with an entry.
- **M0 and M0b**: the loose ends, and the own icon, screenshot and README.
- **Tasks (M1a, partly)**: imports, convert, export, scrub and hash run as tasks with progress
  bars; waiting, moving earlier, cancelling and hashing tried, the rest in "Waiting for the Mac".
- **Messages (M1a+, partly)**: the same message is counted instead of repeated, and the Messages
  page lists every message since the start.

## Waiting for the Mac

What cloud sessions can't do: real drives, consoles, the macOS build and its look. In this order;
each item is ticked off here (and gets its test log entry) when done.

1. ~~**M1a, import as tasks** (#5)~~: done 2026-10-31, see the test log (read-only drive tried
   with a read-only disk image; the NTFS wording of the message is still untested).
2. ~~**M0b, own face**~~: done 2026-10-31, see the test log.
3. **M1a, convert, export, scrub and hash as tasks** (#10, merged before the test too): in the
   Toolbox convert a game into RVZ and cancel it halfway (no file left); export two games
   from WiiGames at once (the second waits), move a waiting one earlier; scrub a Wii game with an
   update partition, then cancel a second scrub (the game must still start in USB Loader GX);
   check the hash of a known good dump. Test log entry.
   Partly done 2026-10-31 (see the test log): waiting, moving earlier, cancelling and hashing
   work, and a known good dump (Worms 3D) gives "Hash match". Pausing an import by hand and
   resuming it works (2026-11-09, on a FAT32 disk image). Still open: export, convert, scrub,
   and the hash of a GameCube game on two discs (both discs are hashed since 2026-11-09; the
   message names the disc that doesn't match).
4. **Every later M1a and M1a+ pull request**: try it on the Mac before merging (the stay-awake
   one with a long import and the Mac left alone until it would sleep).
5. **M1c, copy all to drive** (#41): in the library search a small Wii or GameCube game that
   isn't on WiiGames, "Copy all to drive" → one import task; search one that is there → the
   button is gone. Test log entry.
8. **M2, game databases** (#48 and #53 on top of it; numbered 8 because the branches of #11,
   #13 and #49 bring items 6 and 7): merged only after Claude tried them on the Mac (the
   maintainer's decision, 2026-11-10). A portable debug build of #53's branch (it has #48 too)
   with the real ROM folder as the only ROM source; the scan only reads it.
   - #48: the library shows release names instead of file names, "✓ good dump (No-Intro)" and
     the checksums in the tooltip; a zip with one ROM next to the same ROM unpacked is one entry
     with two copies; `library-checksums.json` is written, and after renaming a ROM the next
     scan doesn't read it again (fast) and still knows it.
   - #53: a start with the network writes `databases/no-intro/state.json` in the data folder
     with six ETags and shows no message (libretro-database is still at the shipped version); a
     start without the network shows nothing either; a hand-made `databases/no-intro/gba.tsv`
     with a later version in its first line (e.g. `2099.01.01`) and one line with a real ROM's
     checksums under a made-up name shows that name after a restart, and the real one again
     once the file is deleted.
   Test log entry; then Claude merges #48, retargets #53 to `main` and merges it.

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

M1b and M1d keep their names but come after M5b (decided 2026-11-09): M2b, M3, M4, M5 and M5b
all build on M2's databases and can be built and tested mostly without the Mac, while M1b formats
real disks and needs the Mac for almost every step; its trigger (reformatting WiiGames) is done
by hand already. Only M6a needs M1b and M1d, so they come right before it.

### M0 · Loose ends (S) – done

- Writing actions check first whether the drive is writable: one clear message ("WiiGames is
  read-only (NTFS, which macOS only reads)") instead of one per file; the drive card shows it.
- The same file in several sources is one entry with "2 copies".
- A drive's games and homebrew are tabs below a bar with the way back to My Consoles.
- The library shows the last scan at once and scans again in the background. (Ignoring folders
  turned out not to be needed: the media folders scan in under a second; the slow part was the
  cold hard disk.)

### M0b · Own face: icon, screenshot, README (S–M) – done

- Icon: the letters "UGBM" in a 2×2 grid, white on a blue–violet rounded square,
  `assets/icon.svg`, chosen by the maintainer instead of drafts. `just icons` (see BUILDING.md)
  makes every size from it: the PNGs, the `.ico`, the macOS iconset and `.icns`, the Linux
  AppDir sizes, the window icon and the About page (#3; the `.icns` made on the Mac).
- Screenshot `assets/screenshot.png`: the My Consoles page with real consoles and drives, used in
  the README and the Linux `metainfo.xml`.
- README: injects section, phase table replaced by a short overview linking here, every project
  it names linked.
- Looked at on the Mac 2026-10-31 (see the test log). At 16 px the four letters blur; the
  maintainer keeps it that way (a 16 px variant with only "U" would be the fix if it bothers).

### M1a · Task manager with progress bars (M) – before M1

**Pull requests, in this order** (each leaves `main` usable):
1. **Core** (#4, merged): `src/tasks/` without iced – tasks with resources and one
   queue per resource, progress with speed and time left, cancel and pause (`TaskControl`),
   retry, reordering, history. Tested, not used by the UI yet.
2. **Import as tasks** (#5, cloud session 2026-10-30, merged): the import queue
   becomes tasks, with progress bars and the bar at the bottom of every page; replaces
   `LongOperationKind::Import`. Imports start when added (no "Start importing" any more); pause,
   cancel (removes what was written) and retry per task; the Tasks page already lists every
   task and the history. Still to try on the Mac with real games (test log).
3. **Convert, export, scrub, checksums as tasks** (#10, cloud session 2026-10-30, merged):
   `LongOperations` is removed; waiting tasks can be moved earlier
   or later on the Tasks page. Still to try on the Mac (test log).
4. **Stay awake** while a task is running (macOS, Windows, Linux) (#7, cloud session
   2026-10-30, merged 2026-10-31): `src/util/keep_awake.rs`, held while `tasks.is_busy()` (a paused task lets go).
   Tried on the Mac 2026-10-31: `caffeinate` runs only while a task runs; the long sleep test
   (item 4 in "Waiting for the Mac") is still due.
5. **Unfinished tasks after a restart**: listed again, to resume or drop. Split off from 3,
   which was big enough; needs the jobs saved in the data folder.

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

First step (#20, 2026-10-31): the same message again is counted in one toast ("245×") instead of
shown again; the log file (`log.txt`, already there) gets every one. Merged; works on the Mac
(2026-10-31, see the test log).

Messages page (#24, 2026-10-31, on top of the first step, merged): every message since the start (the
last 1000; runs of the same one are one entry), newest first, a filter "Only warnings and
errors", a search field, "Open log file". Works on the Mac (2026-10-31), the search not tried
yet. Still open: the source (task, console, drive) with a link to it, grouping different errors
of one task, log levels with debug lines, export.


### M1 · Install on the 3DS over Wi-Fi (M)

Not built yet, and vcinject has it (compared 2026-11-10, see "From vcinject" below): **the first
install of vcinject Link** on a 3DS that doesn't run it yet – the app serves the CIA over HTTP and
shows a QR code that FBI scans (vcinject's `serve.py`). Without it a new 3DS still needs vcinject
once. The same way installs any CIA on a console without vcinject Link. And **installing on every
online 3DS at once**, not one console per click.

- "Install" on a CIA in the library: choose the 3DS; it runs as a task (M1a), with progress on the
  computer and on the 3DS.
- Delete and launch titles from the console's title list.
- Offer the update of vcinject Link when it is older than the app expects.

### M1c · Copy many games to a drive (S)

- "Copy to drive" in the library for a selection, a platform or a whole source folder, not one game
  at a time; files over 4 GiB are always split for FAT32 (`.wbfs` + `.wbf1` …), as the drive import
  already does. Found when copying the WBFS backup to the freshly formatted WiiGames drive, where
  the Finder refuses files over 4 GiB.
- First step (#41): "Copy all to drive" in the library's title bar copies the Wii and GameCube
  discs shown (so a platform, or what the search finds), skipping those already on the drive and
  those only in the cloud, naming the files it can't read, and copying nothing when the files
  together are bigger than the drive's free space. Games already waiting in the task list are
  skipped (#36). Still open: a selection, a whole source folder.

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
- From vcinject (compared 2026-11-10), for the ROMs of the older consoles (Game Boy to Mega Drive):
  - The library remembers each file's checksum (CRC32, SHA-1) and cartridge serial, so a renamed or
    moved file is recognised without being read again (today: name and size), and the game info
    shows them with a link to the release's entry in the libretro database.
  - Zipped ROMs (one game per .zip) are read like unpacked ones.
  - **Artwork**: box art, title screens and logos from the libretro thumbnails, matched by
    checksum; ROM hacks and games missing there from RetroGamesNexus, Pokémon hacks by the name in
    their own text (vcinject's `romtext.py`); GameTDB for 3DS and Wii U retail titles. Covers are
    kept for good and refreshed by ETag; a first download of all covers shows progress and can
    carry on at the next start.
  - **Descriptions** in the user's language from RetroGamesNexus or Wikipedia (`about.py`), also
    for games that are missing.

### M2b · Explain and repair bad dumps (M) – after M2

Today the game info's "SHA1" button only says "Hash mismatch" (a dump's SHA-1 against the Redump
list). A hash can't repair anything, but most mismatches aren't damage, and real damage can often
be fixed from a second copy, which the library knows about.

- **Say why it doesn't match**, instead of only "mismatch":
  - *shortened*: the update partition was removed or the game was scrubbed. The game works; the
    dump can't match Redump again without that data.
  - *damaged*: each 32 KiB block of a Wii partition carries SHA-1s of its data (the H0–H3
    tables); checking them names the bad blocks, as Dolphin's "Verify" does. GameCube discs have
    no such tables, so for them only the whole hash is known.
  - *other version*: a hack, another region or revision; with M2 the databases say which.
- **Repair from another copy**: when the library has the same game twice (e.g. WiiGames and
  gamestation), take the good blocks from each, and keep the result only if its hash is in the
  Redump list. Never overwrite the original; write a new file. Without a second copy the only way
  is a new dump from the disc.
- Maybe later: put a removed update partition back from another game of the same region whose
  partition is byte-identical.
- Both discs of a GameCube game on two discs (`game.iso` and `disc2.iso`) are hashed, and the
  message names the disc that doesn't match (2026-11-09).
- A Redump-exact dump does match: Worms 3D [GWME51] (GameCube ISO on WiiGames, the same as the
  Redump-named one on gamestation) gives "Hash match" (2026-10-31). Every WBFS on WiiGames tried
  gave "mismatch". Read with nod on 2026-10-31: 32 of the 48 Wii WBFS on WiiGames have no update
  partition (*shortened*); the message now says so (#38). The other 16 have one and still don't
  match, e.g. Disney Planes Fire & Rescue [SQQPVZ]: SHA-1 `f0bce155…` against Redump's
  `28cacd91…` (Europe) – damaged, zeroed update partition or a nod reconstruction issue.
- The H0–H3 check doesn't work on WBFS as it is: nod's `PartitionOptions::validate_hashes`
  reports a bad H3 or H0 hash in the first megabytes of almost every WBFS on WiiGames (tried on
  5 on 2026-10-31), most likely where WBFS left out unused block groups. It has to skip the
  groups the image doesn't store, or it only works on full ISO/RVZ dumps.
- **ROMs of the handhelds** (from vcinject's `dumpdb.py`, compared 2026-11-10): besides the
  No-Intro checksums, Game Boy, Color and Advance ROMs are checked against the database of
  [FlashGBX](https://github.com/lesserkuma/FlashGBX) (the cartridge by its header, then CRC32 and
  size). A ROM is *verified*, *differs from the known dump* or *unverified* (in no list: hacks,
  translations, homebrew).

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
- Language versions and copies of a game are one entry too ("3 versions"), with a menu to pick
  the one to use and the regions as small labels on the cover (DE, US/EU, JP, "DE×2"), as vcinject
  does. Copies of the same release are flagged, bit-identical or not.

### M5 · Missing games and collections (M)

- Per platform: have / missing, from the databases.
  As in vcinject: a region filter, "owned X of Y",
  the covers of missing games greyed out, *Owned first* and *Only series I own*.
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

### M5b · File management (M–L)

Builds on identified games (M2) and one entry per game (M4). Everything that moves, renames or
deletes files runs as a task (M1a) and shows a preview first.

**Folder structure, configurable**
- Per ROM source a **layout** as a template with placeholders, e.g.
  `{platform}/{title} [{region}]/{title}.{ext}` or `{platform}/{series}/{title} ({id})`, with
  presets for common layouts (No-Intro names, one folder per game, USB loader layout `wbfs/`).
  Separate layouts for ROMs, injects, saves and archives (RAR, ZIP) where wanted.
- **Organize** a source into its layout: the app shows a dry run – what moves where,
  which names change, conflicts (two files for one target, names too long for FAT32) – and only
  runs after confirmation. Every run is logged and can be undone from that log.
- **Clean up**: find duplicates (same checksum), empty folders, leftovers (`.part`, unpacked RAR
  caches, macOS `._` files), unidentified files; each as a list to review, nothing is deleted
  silently. Deleted files go to the system's trash where there is one.
- **From vcinject** (compared 2026-11-10): *Zip* packs every unpacked ROM into its own .zip next
  to it (the original is deleted only after its zip was verified); *Names* renames identified ROMs
  to their release name; a ROM can be marked as *self-dumped*.
- The layout of devices (drives, SD cards) stays what the console expects (M1b); the layout here
  is for the collection on the computer.

**Backup rules**
- Rules per platform, source, collection or single game: *what* (ROM, saves, injects), *where*
  (another ROM source, an external drive, a network share), *how often* (on change, daily,
  weekly, by hand) and *how many* versions to keep (e.g. the last 10 saves, every save of the last
  30 days).
- Saves are versioned (timestamp, console they came from, checksum); ROMs and discs are only
  copied once and checked by checksum afterwards.
- A status per game and per rule: backed up, outdated, never; a report of what isn't backed up
  anywhere (e.g. "37 games exist only on gamestation").
- Restoring picks a version and a target (a source folder, a drive, a console over Link).

**ROMs and saves in the game view**
- The game view (M4) gets the tabs **Files** (ROMs, discs, injects with source, size, checksum,
  identification), **Saves** (all versions per console, with date and size; back up, restore,
  export, delete, compare) and **Backups** (which rules cover the game, last run, where the copies
  are).
- From there: set the game's own rule, move a file to another source, mark one copy as the main one.
- Saves come from the consoles (3DS M6, Wii M7, Wii U M8) and from save folders on drives
  and SD cards; until then the tab shows the save files found in the sources.

### M1b · Prepare SD cards and drives (M) – after M5b, before M6a

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

### M1d · SD cards: manage, back up, set up (M) – after M1b

The SD card of a 3DS, a Wii or a Wii U becomes a device in My Consoles, like a drive: the app
sees what is on it, keeps backups of it and sets up new cards. Builds on M1b (format and layout).

**Recognising a card and its console**
- By its content, not its name: 3DS (`Nintendo 3DS/`, `boot.firm`, `luma/`, `3ds/`, `cias/`),
  Wii (`apps/`, `private/wii/`, `wad/`, BootMii's `bootmii/`), Wii U (`wiiu/`,
  `wiiu/environments/aroma/`, `wiiu/apps/`, `wiiu/plugins/`), or a mix (a Wii U card often
  holds vWii homebrew too).
- The card remembers its console (name in My Consoles), so a card in the computer shows up under
  the right console.

**Managing**
- What is on it, grouped: CFW or environment and its version (Luma3DS, Aroma), homebrew apps,
  plugins, CIAs and WADs waiting to be installed, saves, free space; the M1b check of file system
  and cluster size.
- Update homebrew from its official source (Open Shop Channel, Universal-DB, the projects'
  GitHub releases), remove apps, move games and apps between card and library.

**Backing up**
- One backup per card and date in the data folder (or a chosen folder), as a plain copy of the
  files; restoring to the same or to a new, larger card (moving to a bigger card is the common
  case).
- **Irreplaceable files** are marked and their absence warned about, because without them a
  broken console can't be restored:
  - 3DS: the NAND backup and `essential.exefs` from GodMode9 (`gm9/out/`); the content of
    `Nintendo 3DS/` is encrypted for this one console and only works on it.
  - Wii: BootMii's `nand.bin` and `keys.bin`.
  - Wii U: the NAND, `otp.bin` and `seeprom.bin` dumps.
- These files belong to the console and stay on the computer; they are never uploaded or shared.

**Setting up**
- A new card for a console that is already modded: M1b's layout plus the homebrew the console
  uses, or the content of a backup.

**Models and revisions**
- 3DS family: Old 3DS, Old 3DS XL and 2DS take a full-size SD card; New 3DS, New 3DS XL and
  New 2DS XL a microSD card (behind a screwed cover on the New 3DS). The app asks for the model
  once and remembers it with the console.
- Wii: RVL-001 (with GameCube ports), RVL-101 "Family Edition" (no GameCube ports, so no
  Nintendont from discs), RVL-201 Wii mini (**no SD slot**: USB only, so its "card" is a USB
  stick).
- Wii U: one SD slot for Wii U and vWii homebrew; game drives are WFS (see M1b).

### M6a · Softmod wizard (L)

A wizard in My Consoles that takes an unmodded console to custom firmware or the Homebrew Channel,
**following the community guides step by step instead of replacing them**:
[3ds.hacks.guide](https://3ds.hacks.guide), [wii.hacks.guide](https://wii.hacks.guide) and
[wiiu.hacks.guide](https://wiiu.hacks.guide). Order: 3DS first (with M6), then Wii (M7) and Wii U
with vWii (M8).

**How it works**
- The guides stay the reference: every step names and links its page in the guide; the wizard
  doesn't copy their text (they change, and they are their authors' work). If the guide and the
  wizard disagree, the wizard says to follow the guide.
- The computer's part is automated: preparing the SD card (M1b, M1d), downloading the files a
  step needs from their official sources at that moment, checking that they arrived complete, and
  later reading back what the console wrote (backups).
- The console's part is shown, not done: the wizard says what to do on the console, links the
  guide's page and waits for "done" or reads the result from the card.
- It can be left and resumed: the state is read from the card (e.g. `boot.firm` and `luma/` there:
  boot9strap is installed).
- Nothing that isn't freely distributable is bundled; no keys, no firmware, no piracy tools (the
  guides leave those out too).

**Step 1: which console exactly** – the method depends on model, revision, system version and
region, so the wizard asks first (with pictures and where to read it on the console):
- 3DS: model (Old 3DS / Old 3DS XL / 2DS / **New 3DS / New 3DS XL / New 2DS XL**), system
  version (e.g. `11.17.0-50E`) and region. New models have the faster CPU and the microSD slot;
  some methods work only on New models (super-skaterhax: New 3DS and New 2DS from 11.15.0 to
  11.17.0), others on all models (MSET9).
- Wii: RVL-001, RVL-101 Family Edition or RVL-201 Wii mini; System Menu version and region. The
  guide picks the exploit from that: e.g. str2hax (needs internet, no SD card), Bluebomb
  (Bluetooth; the only way on the Wii mini, which the guide warns is easy to brick).
- Wii U: system version and region; the guide's way to Aroma (e.g. the browser exploit), then
  PayloadLoader for a permanent setup, then vWii through wii.hacks.guide.

**Step 2: the method** – chosen from a table of model × version → guide page, kept as a data file
in the app and checked against the guides before each release. A console that doesn't fit the
table (a newer system version, an unknown revision) gets no guess: the wizard sends the user to the
guide's start page.

**Step 3: backups first** – as soon as the console can make them (GodMode9's NAND backup and
`essential.exefs`, BootMii's NAND backup, the Wii U's NAND, OTP and SEEPROM), the wizard asks for
them and copies them into the console's backups (M1d). It doesn't go on without them.

**Step 4: afterwards** – the usual homebrew (from M1d) and the companion app (vcinject Link on the
3DS, later the Wii and Wii U ones), then the console is added to My Consoles.

**Safety**
- The app never writes to a console's NAND and never runs a step that can brick it; those stay on
  the console, done by the user with the guide open.
- Warnings the guides give (e.g. "don't update", "don't follow video guides for the Wii mini") are
  shown at the step they belong to.
- Tested on real consoles of each model the table names before it is released for that model;
  every test goes into the test log.

### M6 · The rest of the 3DS (M–L)

- Saves: back up and restore.
- HOME Menu: folders and order.
- Universal-DB: homebrew store.
- Injector: build 3DS injects from the library, first through vcinject as a command-line sidecar,
  later ported to Rust.

**From vcinject** – what it does today and this app doesn't (compared 2026-11-10 with vcinject's
`main`, which holds all of its branches; "M1" and "M2" items are listed in those milestones):

- **Status report per console** (`status.py`): Luma3DS, vcinject Link and the installed homebrew
  against their newest releases, the sound firmware `dspfirm.cdc` (RetroArch forwarders need it),
  free space on the SD card; *Update* per finding and *Update all*, including Luma3DS's
  `boot.firm`. The report is mirrored on the 3DS's screen (Link's `report` command, which the
  client has but nothing uses). After updates that need a restart the app restarts the 3DS,
  reconnects and says whether everything worked.
- **Saves** (`saves.py`, `disa.py`): one zip per console and title in the data folder; restore
  picks one. Restoring is untried on hardware in vcinject too.
- **Titles**: save a title's icon and banner; tell official Virtual Console titles and injects
  apart from other games (open in vcinject as well).
- **HOME Menu** (`homemenu.py`): read the layout as on the console, arrange games and apps on the
  SD card and move them into the existing folders; *Save to 3DS* backs up the old layout, writes
  the new one, restarts the 3DS and checks every icon. Folders, system apps and the Game Card
  can't be written while the system runs: vcinject plans a GodMode9 script for that (its
  `docs/TODO.md`); the dead ends it tried are written down there.
- **Homebrew**: besides all of Universal-DB (over 200 apps with a .cia, searchable, with
  screenshots, the version on each 3DS and one button that installs or updates), a short
  *Recommended* list straight from the authors' GitHub releases, only projects with an open
  licence (DSP1, FBI Reloaded, Universal-Updater, Checkpoint, PKSM, ftpd, Anemone3DS), and
  **News**: the latest releases of those, Luma3DS and this app.
- **Injector**, what belongs to it besides building the CIA:
  - Systems: GBA natively (AGB_FIRM; save types detected, 1 Mbit flash), SNES (snes9x), NES
    (VirtuaNES), Game Boy / Color and Mega Drive (RetroArch) as forwarders. Nintendo's own
    GB/GBC emulator is experimental in vcinject (a fresh install lacks `sav.dat`), its NES one
    planned.
  - **VC Console Look**: the 3D banner per console with the game's picture on the cartridge, the
    "Virtual Console" plate in the 13 console languages, the jingle and the VC icon frame
    (`vcbanner.py`, `cgfx.py`); Game Boy looks derived from the GBA one.
  - **Templates** come from titles the user owns, never from the app: a setup checklist that says
    what each system needs, checks it on every start and sets up again what is missing or damaged
    (`health.py`), and export/import of all templates as one private backup (`backup.py`).
    vcinject's idea to read the looks from titles installed on the 3DS over Link is open there too.
  - **Title IDs**: a register of every build so that no ID is used twice (`registry.py`); the
    titles on every console that runs vcinject Link count as taken, and for a 3DS without it its
    `ticket.db` can be imported (`tickets.py`). Rebuilding a game keeps its ID, so the save stays.
  - `makerom` and `bannertool` are needed as long as vcinject is the sidecar; its installers
    bring them along.
  - Per build: title, publisher, release year, own icon and banner, save type, LCD ghosting,
    brightness, plate language; a preview of the HOME Menu icon and the rotating banner.
- **Console pictures**: a photo of the exact model in the colour chosen per console (the 3DS
  can't tell its colour), from Wikimedia Commons, public domain or CC0, downloaded on first use
  (`consolephotos.py`).
- **vcinject Link's source** stays in vcinject (`companion/`, built with devkitARM); this app
  ships the built CIA (#11). Version 13 doesn't have the [Link protocol](LINK_PROTOCOL.md)'s
  streams (commands 19–21) and only answers the discovery query `vcinject?`: a version 14 is
  needed for both, see "Open questions".
- **Views**: the size of the cards changes without steps (slider, Ctrl + mouse wheel), Cover Flow
  next to cards and table.

### M7 · Wii (L)

- Companion app (libogc) speaking the Link protocol: title list, WAD install, saves.
- Drives: FAT32 guidance and formatting help (the WiiGames drive is being reformatted).

### M8 · Wii U (L)

- Companion app for Aroma (`.wuhb`) with WUP installs like WUP Installer GX2, title list, saves.
- Injector modelled on UWUVCI (GPL-3.0, so compatible).

### M9 · Server version in Docker with a web UI (L)

The app as a service on a NAS or home server, in the style of Sonarr/Radarr (the *arr apps):
runs all the time, is used in the browser, watches the collection and does the backup rules on
its own.

- **One core, two faces.** Library, devices, tasks, databases, file management and the Link
  client move into a core crate without iced; the desktop app and the server both use it. This
  split is prepared in every milestone before: new logic goes into the core, not into the UI.
- **Server**: a headless binary (`ugbm-server`) with an HTTP API (REST + WebSocket/SSE for task
  progress and notifications) and the web UI served by the same process. API documented
  (OpenAPI), so scripts and other tools can use it.
- **Web UI** like the *arr apps: sidebar (Library, Missing, Collections, Devices, Tasks,
  Activity/History, Settings, System), tables with filters and bulk actions, a queue page,
  the log and system status (version, disk space, database age, health checks such as "ROM source
  not reachable").
- **Docker image** in the usual home server style:
  - `/config` for settings, databases and the log; ROM sources and backup targets as volumes
    (e.g. `/data/roms`, `/data/backups`), paths mapped in the settings.
  - `PUID`/`PGID`, `TZ`, `UMASK`; port `8765` (to be chosen); multi-arch (amd64, arm64);
    a healthcheck; images on GHCR, built in CI for every tag.
  - A `docker-compose.yml` as an example in the docs.
- **Security**: login (form or basic auth), an API key for scripts, "no login on the local
  network" as an option like the *arr apps; meant for the home network, HTTPS through a reverse
  proxy.
- **What changes in a container**:
  - Finding consoles needs UDP broadcast/mDNS: `network_mode: host`, or consoles entered by IP.
  - Formatting disks (M1b) isn't offered; the server works with folders and shares.
  - Keeping the computer awake isn't needed; notifications go to the web UI and optionally to
    webhooks (Discord, ntfy, Gotify, e-mail), as the *arr apps do with "Connect".
- **Scheduler**: backup rules (M5b), database updates (M2) and library scans run on their own,
  with the scan watching the folders for changes.
- Tested in CI with the image started and the API called; a test log entry (see TESTLOG.md) on a
  real NAS before the first release.
- A **command line** on the same core, for scripts without the server: vcinject has one (`build`,
  `verify`, `library`, `doctor`, `3ds install` …), this app none.

## Open questions

- How large the database snapshot gets, and whether it needs Git LFS.
- How collections are shared between people.
- Whether a source for series exists that is good, open and usable offline (see M5).
- Formatting FAT32 above 32 GB with Windows' own tools: which versions allow it (see M1b).
- Which template syntax for folder layouts, and how undo works when files changed after a run
  (see M5b).
- Where save versions live: next to the game in the ROM source or in one save store (see M5b).
- Web UI: which framework (server-rendered with htmx, or a single-page app), and whether the
  desktop app can also be reached through it (see M9).
- How much of the core can be split from iced without slowing down the milestones before M9.
- Softmod wizard (M6a): the method table per model and system version has to be checked against
  the live guides; the cloud sessions can't reach the guides (network settings), only search
  results about them (2026-10-30). Their pages are linked, not copied; whether short step titles
  may be shown depends on the guides' licence, to check.
- Wii mini (M1d, M6a): the app has to handle a USB stick in place of the SD card.
- vcinject Link (M1, M6): where its next version is developed – in vcinject's `companion/` as
  until now, or moved here and renamed for the protocol's `ugbm?` discovery – and whether vcinject
  itself goes on next to this app or only as the injector's sidecar.
