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
- **[Test log](TESTLOG.md)**: every test on real hardware is written down there – device, what,
  who, when, how, why and the result. From now on each milestone counts as done only with an entry.

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
4. **Every later M1a pull request**: try it on the Mac before merging (the stay-awake one with a
   long import and the Mac left alone until it would sleep).

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
1. **Core** (`claude/m1a-task-core`): `src/tasks/` without iced – tasks with resources and one
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
4. **Stay awake** while a task is running (macOS, Windows, Linux).
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

### M1d · SD cards: manage, back up, set up (M)

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
