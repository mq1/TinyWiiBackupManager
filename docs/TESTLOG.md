# Test log

Every test on real hardware or real data gets an entry here: which device, what was tested, by
whom, when, how, why, and what came out. CI covers the code; this log covers what only a real
console, a real drive or a real collection can show.

## How to write an entry

- One entry per test session and device; newest on top.
- **Result**: ✅ works, ⚠️ works with limits, ❌ fails. A failure links the issue or the roadmap
  milestone that takes it up.
- Name the build (version or commit) and the operating system of the computer.
- Name devices as they appear in My Consoles (e.g. the 3DS "petter1", the drive "WiiGames").
- Only what was actually done and seen; nothing from memory or expectation.

### Template

```markdown
### YYYY-MM-DD · <device> · <what, in a few words>

| | |
| --- | --- |
| **Device** | <console / drive / card, model, firmware or file system> |
| **What** | <feature or milestone tested> |
| **Who** | <tester> |
| **When** | YYYY-MM-DD, build <version or commit>, <OS of the computer> |
| **How** | <steps, data used (how many games, which sources)> |
| **Why** | <new feature, bug fix to confirm, regression check, before a release …> |
| **Result** | ✅ / ⚠️ / ❌ <what came out; numbers, messages, times> |
| **Follow-up** | <issue, roadmap milestone, or "none"> |
```

## Devices

| Name | Kind | Details |
| --- | --- | --- |
| petter1 | 3DS | runs vcinject Link |
| WiiGames | Wii/GameCube USB drive | was NTFS (macOS reads only), being reformatted to FAT32 (M1b) |
| gamestation | drive | ROM source |
| iCloud collection | ROM source | files only in iCloud aren't downloaded |

## Entries

### 2026-10-28 · WiiGames · importing Wii games to the drive

| | |
| --- | --- |
| **Device** | WiiGames, Wii/GameCube USB drive |
| **What** | Import of Wii games to a drive; copying a WBFS backup with the Finder |
| **Who** | petter1 |
| **When** | 2026-10-28, macOS |
| **How** | 116 Wii games imported to the drive; WBFS backup copied to the freshly formatted drive |
| **Why** | Fill the drive for the Wii |
| **Result** | ⚠️ While NTFS, every write failed with "Read-only file system", one toast per file. The computer went to sleep from idleness during the long import. The Finder refuses files over 4 GiB on FAT32. |
| **Follow-up** | One message for a read-only drive (M0, done); stay awake while data moves (M1a); notifications grouped (M1a+); copy many games with splitting (M1c); formatting (M1b) |

### 2026-10-28 · petter1 · My Consoles and titles

| | |
| --- | --- |
| **Device** | petter1, 3DS with vcinject Link |
| **What** | Discovery in My Consoles, title list with names and icons |
| **Who** | petter1 |
| **When** | 2026-10-28 |
| **How** | App started with the 3DS in the same Wi-Fi |
| **Why** | First version of the Link client |
| **Result** | ✅ Found; titles with names and icons. Connecting needed more patience than first built in (fixed the same day). |
| **Follow-up** | Install over Wi-Fi (M1) |

### 2026-10-28 · iCloud collection, gamestation · Library

| | |
| --- | --- |
| **Device** | iCloud collection and the gamestation drive as ROM sources |
| **What** | Library from several ROM sources |
| **Who** | petter1 |
| **When** | 2026-10-28, macOS |
| **How** | Both added as ROM sources, library scanned |
| **Why** | First version of the Library page |
| **Result** | ✅ Media folders scan in under a second; the slow part is the cold hard disk. The same file in several sources showed twice (fixed: one entry with "2 copies"). Files only in iCloud aren't downloaded. |
| **Follow-up** | Show the last scan at once (M0, done) |
