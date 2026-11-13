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
| Petter1 | 3DS XL | runs vcinject Link; a second console, named like the 3DS in My Consoles |
| WiiGames | Wii/GameCube USB drive | FAT32 (was NTFS, which macOS only reads) |
| gamestation | drive | ROM source; HFS+, read-only on Linux |
| iCloud collection | ROM source | files only in iCloud aren't downloaded |
| WIIRO | drive | read-only FAT32 disk image (`hdiutil attach -readonly`), for read-only tests |

## Entries

### 2026-11-13 · gamestation · RAR and 7z through 7-Zip

| | |
| --- | --- |
| **Device** | gamestation, USB HDD, HFS+, mounted read-only on Linux |
| **What** | ROMs in 7z/RAR, names of title IDs from the packs' Loadiine archives (`claude/m2-7zip-archives`) |
| **Who** | Claude, on the maintainer's Linux computer |
| **When** | 2026-11-13, Arch Linux, debug build, 7-Zip 26.06 (`/usr/bin/7z`) |
| **How** | Portable build with `gamestation/Games` as the ROM source, started without a network; `library.json` and `library-archives.json` read after the scan |
| **Why** | 43 items were only a title ID; their names are in the packs' Loadiine RARs |
| **Result** | ✅ 42 of 43 named ("0005000010106006" → "PocketNES: Roms S Pack 1", all 25 Goomba Color GBC titles), from 5 Loadiine RARs; left: `NDS/0005000010102044.rar` (NUS, no readable name). Both `.7z` GBA hacks read as ROMs (checksums, serial). 7 archives opened by 7-Zip, the thousands of inject RARs named after a title ID not. |
| **Follow-up** | `unar`/`lsar` fallback; the same on the Mac with Homebrew's `7zz` |

### 2026-11-13 · gamestation, WiiGames · names for discs and title IDs (#58)

| | |
| --- | --- |
| **Device** | gamestation, USB HDD, HFS+, mounted read-only on Linux; WiiGames (`wbfs` folder only), FAT32 |
| **What** | GameTDB names for discs without a Redump name, folder names for items that are only a title ID (#58) |
| **Who** | Claude, on the maintainer's Linux computer |
| **When** | 2026-11-13, #58's branch on top of #56, Arch Linux, debug build |
| **How** | Portable build with `gamestation/Games` and `WiiGames/wbfs` as ROM sources (3,055 items); `library.json` compared with a scan of the build before #56 and #58 |
| **Why** | Many discs showed the header's internal name ("AITD5_111_080403_EU"), 100 items only a title ID |
| **Result** | ✅ Of 133 discs and archives with a game ID, 98 show the Redump name, 24 hacks and custom discs GameTDB's ("Guitar Hero Queen" → "Guitar Hero III Custom : Queen", "New SUPER MARIO BROS. Wii" `SMNEZY` → "Super Mario Skyland"), the rest the disc's. 57 of 100 bare title IDs named from their folders ("0005000010104857" → "the legend of zelda ocarina of time usa", "0005000010102139" → "SMSAdvance - Master System Roms A-G v4"); 43 left, in packs of several titles or straight in a platform's folder. |
| **Follow-up** | M2: names from inside the RARs (7-Zip) |

### 2026-11-13 · gamestation · zips named like a ROM (#59)

| | |
| --- | --- |
| **Device** | gamestation, USB HDD, HFS+, mounted read-only on Linux |
| **What** | A zip with a ROM's name (`game.gba`) read as the ROM in it (#59) |
| **Who** | Claude, on the maintainer's Linux computer |
| **When** | 2026-11-13, build `8442141f` (#59's branch), Arch Linux, debug build |
| **How** | Portable build with `gamestation/Games/GBA` as the only ROM source, new data folder (nothing known yet) |
| **Why** | 9 such files weren't identified in the test of #48 |
| **Result** | ✅ All 9 identified by SHA-1, e.g. `mega-man-battle-network-2-gba.gba` → "Mega Man - Battle Network 2 (USA)", `spider-man-2-gba.gba` → "Spider-Man 2 (USA, Europe)" |
| **Follow-up** | none |

### 2026-11-13 · gamestation, WiiGames · Wii and GameCube discs named from Redump (#56)

| | |
| --- | --- |
| **Device** | gamestation, USB HDD, HFS+, mounted read-only on Linux; WiiGames (`wbfs` folder only), FAT32 |
| **What** | M2: Redump names for Wii and GameCube discs by game ID (#56) |
| **Who** | Claude, on the maintainer's Linux computer |
| **When** | 2026-11-13, builds `558570e4` and `deac4056` (#56's branch), Arch Linux, debug build |
| **How** | Portable build with `gamestation/Games` and `WiiGames/wbfs` as ROM sources (3,055 items, 133 Wii/GameCube discs and archives with a game ID); `release` in `library.json` compared with the names GameTDB gives the same game IDs. |
| **Why** | #56 waited for a scan of the real collection |
| **Result** | ⚠️ then ✅ The first build named 112 of 133, but 14 wrongly: custom discs and mods keep another game's code under their own maker code (`RMCE13` Wiimms MKW Fun → "Mario Kart Wii (USA)", `RQQE52` Guitar Hero III Custom: Queen → "Backyard Football '09 (USA)", `SRGP01` NSMBW 2 Reggie → "Rango (Europe)"). Fixed in `deac4056` (no Redump name when GameTDB names the whole ID differently): 98 of 133 named, exactly the 14 hacks dropped; official games keep theirs, also "Pokémon"/"PokéPark" (accents) and the Japanese Pokémon Battle Revolution. |
| **Follow-up** | none |

### 2026-11-12 · gamestation · game databases: ROMs identified by checksum (#48, #53)

| | |
| --- | --- |
| **Device** | gamestation, USB HDD, HFS+ (journaled), mounted read-only on Linux |
| **What** | M2: No-Intro identification (#48) and refreshing the lists at run time (#53) |
| **Who** | Claude, on the maintainer's Linux computer, maintainer watching |
| **When** | 2026-11-12, build `15e00520` (#53's branch, has #48), Arch Linux, debug build |
| **How** | Portable build (`UGBM-portable`, own data folder) with `Games/GBA` and `Games/GB(C)` of gamestation as the only ROM sources, no drive selected (162 files, 124 ROMs or archives). Then a scratch folder with copies of three ROMs: a zip next to the same ROM unpacked; a ROM renamed and made unreadable (`chmod 000`) before the second scan; a start without network (`unshare -rn`) after deleting `databases/no-intro/state.json`; a hand-made `databases/no-intro/gba.tsv` dated `2099.01.01` with Metroid Fusion's checksums under a made-up name, then deleted again. Results read from `library.json`, `library-checksums.json` and `state.json`. |
| **Why** | #48 and #53 were merged only after a test on the real collection ("Waiting for the Mac" item 8) |
| **Result** | ✅ The scan took a few seconds. 104 ROMs identified by SHA-1 ("good dump"), also files with free names (`kirby-the-amazing-mirror.gba` → *Kirby & The Amazing Mirror (USA)*), self-dumps and zips; 7 ROM hacks and a translation only by serial, shown as "probably Pokemon - FireRed Version (other dump, hack or translation)" and not renamed; 11 not identified, all rightly: a fan translation (`[T-En]`), two `.7z`, a `source.zip`, and 9 files that are zips renamed to `.gba`. Two `… copy.gba` were one entry with a copy, as were the zip and the unpacked ROM. The renamed unreadable ROM was still identified (not read again). With network: `state.json` with six ETags, no list kept (none newer). Without network: no message from the lists (only the app's own update check logs its error, as before), `last_check` 0 so the next start tries again. The hand-made list's name was shown after a restart, the real one again once the file was gone. |
| **Follow-up** | M2: recognise zips by their content, not the extension; `.7z`/RAR via 7-Zip. Which file of two copies leads is the scan's order. |

### 2026-11-09 · FAT32 disk image · no double import (#36), copy all (#41), pause by hand

| | |
| --- | --- |
| **Device** | UGBMTEST, a 3 GB FAT32 disk image (MBR, `hdiutil`), as the drive; WiiGames wasn't connected |
| **What** | #36 (a disc already waiting or being imported isn't added again; Retry refuses it), #41 ("Copy all to drive"), pausing an import by hand |
| **Who** | Claude, on the maintainer's Mac, clicks and screenshots driven by a script |
| **When** | 2026-11-09, `claude/m1c-copy-many` (a87456d6) merged locally with `claude/no-double-import` (e8d2f5a3), debug build in portable mode (own data folder), macOS |
| **How** | Fake GameCube discs (headers only, sparse; 1.4 GB and 8 MB) as the only ROM source. Library: four "Copy to drive" clicks within a second; an import paused by hand to hold the drive, a waiting import cancelled, picked again, Retry on the cancelled one; "Copy all to drive" with a search that needs more than the free space, and with no results |
| **Why** | Try #36 and #41 before merging, without the real drive |
| **Result** | ✅ "Small Bravo is already being imported" (waiting) and "Big Alpha is already being imported" (running), each imported once. Retry → "Small Delta is already being imported", the task stays Cancelled. Pause and Resume work, the file is complete afterwards (1 468 006 400 bytes). "Copying 2 games needs about 2.73 GiB, the drive has 1.60 GiB free: nothing copied", the game already there not counted; no results → no button. Not tried in the window: a "Copy all" that copies (done in a test on a 40 MB FAT32 image: two games copied, the broken file named) |
| **Follow-up** | On WiiGames with real discs (Wii too) and a split over 4 GiB; CI before merging |

### 2026-10-31 · WiiGames, gamestation · why a hash doesn't match (#38)

| | |
| --- | --- |
| **Device** | WiiGames, Wii/GameCube USB drive, FAT32; gamestation (ROM source), answering again |
| **What** | #38: the hash task says why a dump doesn't match; and whether a Redump-exact dump matches the built-in list at all |
| **Who** | Claude on petter1's Mac, the app driven in the background (computer use) |
| **When** | 2026-10-31, `claude/m2b-explain-mismatch` 538e35ab (debug build), macOS 27 |
| **How** | "SHA1" in the game info of Glacier 3 (no update partition) and of Disney Planes: Fire & Rescue (has one, the info shows "Scrub"). Separately, `shasum` of "Worms 3D (USA) (En,Fr).iso" on gamestation (a GameCube ISO named after Redump) looked up in the app's `sha1_list.bin` |
| **Why** | every WBFS on WiiGames gave "Hash mismatch" without a reason |
| **Result** | ✅ Glacier 3: "…: the dump has no update partition (often removed when making a WBFS), so it can't match the disc. The game itself is not affected". Disney Planes: "…: not a known good dump. It may be damaged, another version or changed otherwise; the hash can't tell which". The GameCube ISO's SHA-1 (373505dc…) is in the list; the same file is on WiiGames as Worms 3D [GWME51], and "SHA1" there says "Hash match for Worms 3D! - SHA1 is well known, your dump is perfect". The message names the game twice ("Check the hash of Glacier 3: Hash mismatch for Glacier 3: …") |
| **Follow-up** | the doubled title in the message |

### 2026-10-31 · WiiGames · drive list with a drive that doesn't answer (#35)

| | |
| --- | --- |
| **Device** | WiiGames, Wii/GameCube USB drive, FAT32 |
| **What** | #35: a drive not read within 5 s is listed as "Doesn't answer" instead of emptying the drive list in My Consoles |
| **Who** | Claude on petter1's Mac, the app driven in the background (computer use) |
| **When** | 2026-10-31, branch `claude/drives-not-answering` (debug build), macOS 27 |
| **How** | A freshly signed test build started, so macOS's access prompt for removable volumes was pending; My Consoles opened |
| **Why** | before, My Consoles showed no drives at all for over a minute, even after refreshing |
| **Result** | ⚠️ WiiGames is listed with "Doesn't answer" instead of an empty list. Not tried: that a refresh picks it up once access is allowed (needs a click on the prompt) |
| **Follow-up** | refresh after allowing access |

### 2026-10-31 · WiiGames · a game already on the drive isn't imported again (#33)

| | |
| --- | --- |
| **Device** | WiiGames, Wii/GameCube USB drive, FAT32 |
| **What** | #33: a disc already in its game's folder is left out of an import |
| **Who** | Claude on petter1's Mac, the app driven in the background (computer use) |
| **When** | 2026-10-31, branch `claude/import-keeps-existing-games` (debug build, a temporary test not committed), macOS 27 |
| **How** | `ImportEntry::is_on_drive` against the real drive for Glacier 3 and Xenoblade Chronicles (split WBFS) on WiiGames, and against WIIRO |
| **Why** | a dropped game skipped the check and overwrote the game on the drive |
| **Result** | ⚠️ On WiiGames both are found; on WIIRO, which doesn't hold them, neither (as expected). Not tried in the window: dropping a file onto the Games page (can't be done from the background) |
| **Follow-up** | drop a game that is on WiiGames → "… is already on the drive" |

### 2026-10-31 · WiiGames, gamestation · library scan with a source that doesn't answer (#31)

| | |
| --- | --- |
| **Device** | WiiGames, Wii/GameCube USB drive, FAT32; gamestation (ROM source) hung: every read waited, `ls` didn't return |
| **What** | #31: the library scan gives up a source that reads nothing for 15 s |
| **Who** | Claude on petter1's Mac, the app driven in the background (computer use) |
| **When** | 2026-10-31, branch `claude/library-scan-unanswered-source` (debug build), macOS 27 |
| **How** | Before: `main` 6a7b47d0 left the library at "Scanning…" (the scan thread in `opendir` on gamestation). With #31: app started, `library.json` compared before and after |
| **Why** | gamestation stopped answering during the tests |
| **Result** | ✅ After 15–16 s "/Volumes/gamestation doesn't answer: its games are from the last scan", "Scanning…" ends, `library.json` keeps all 3067 entries (3065 on gamestation). A first version lost gamestation's entries from the cache; caught here, fixed before merging, cache restored from a copy |
| **Follow-up** | none |

### 2026-10-31 · WiiGames · settings with a drive that doesn't answer (#28)

| | |
| --- | --- |
| **Device** | WiiGames, Wii/GameCube USB drive, FAT32 |
| **What** | #28: loading the settings waits at most 3 s for the selected drive |
| **Who** | Claude on petter1's Mac, the app driven in the background (computer use) |
| **When** | 2026-10-31, branch `claude/config-load-slow-drive` (debug build), macOS 27 |
| **How** | A freshly ad-hoc signed `.app` started with `open`, so macOS's prompt for removable volumes was pending; `sample` of the process |
| **Why** | before, the window showed "No drive selected" and no sources until the prompt was answered |
| **Result** | ✅ After about 3 s the title shows WIIGAMES and the library is there; `sample` shows the `read_dir` calls still waiting for the prompt |
| **Follow-up** | none |

### 2026-10-31 · WiiGames · paused task display (#27)

| | |
| --- | --- |
| **Device** | WiiGames, Wii/GameCube USB drive, FAT32 |
| **What** | #27: a paused task shows no speed and counts as paused |
| **Who** | Claude on petter1's Mac, the app driven in the background (computer use) |
| **When** | 2026-10-31, branch `claude/tasks-paused-display` (debug build), macOS 27 |
| **How** | Hash of Xenoblade Chronicles, paused on the Tasks page |
| **Why** | seen in the #10 test: "Paused · … · 106 MiB/s · 32 s left" and "1 task running" |
| **Result** | ✅ Row "Paused · 330 MiB of 7.93 GiB", bottom bar "1 paused" |
| **Follow-up** | none |

### 2026-10-31 · WiiGames · Messages page (#24)

| | |
| --- | --- |
| **Device** | WiiGames, Wii/GameCube USB drive, FAT32 |
| **What** | #24, the Messages page with every message since the start |
| **Who** | Claude on petter1's Mac, the app driven in the background (computer use) |
| **When** | 2026-10-31, `claude/m1a-plus-notification-center` 95ff38ed (debug build), macOS 27 |
| **How** | Two hashes of Glacier 3 (two identical "Hash mismatch"), toast closed, Messages page; "Only warnings and errors", "Clear", "Open log file" |
| **Why** | #24 waits for the Mac test |
| **Result** | ✅ One entry "09:11:38 … Hash mismatch for Glacier 3" with "2×", still there after the toast was closed; the filter keeps it; "Clear" → "No messages yet."; "Open log file" opens `log.txt` in TextEdit. Not tried: a mix of info and error messages for the filter, the 1000-entry limit |
| **Follow-up** | none |

### 2026-10-31 · WiiGames · unfinished tasks after a restart (#13)

| | |
| --- | --- |
| **Device** | WiiGames, Wii/GameCube USB drive, FAT32 |
| **What** | #13, unfinished tasks are offered again after a restart |
| **Who** | Claude on petter1's Mac, the app driven in the background (computer use) |
| **When** | 2026-10-31, `claude/m1a-resume-tasks` c875da5f (debug build), macOS 27 |
| **How** | Two hash tasks (one running, one waiting), then Quit from the menu; again with a hard kill; restart; Tasks page; drop one, resume one, let it finish; `tasks.json` checked after each step |
| **Why** | #13 waits for the Mac test |
| **Result** | ⚠️ `tasks.json` holds both tasks; after Quit and after a kill the app says "2 task(s) weren't finished…" and the Tasks page lists them; drop removes one, resume runs it again, `tasks.json` is `[]` once it ended. **Bug**: no Tasks icon in the sidebar when only unfinished tasks exist, so the page couldn't be reached; fixed in 21a8aec1 and re-checked the same day with that build: after a restart with one unfinished task the icon is there, the page lists it, "drop" empties `tasks.json`. Removing half-written imports, conversions, exports and scrubs not tried (would write to the drive) |
| **Follow-up** | cleanup after an interrupted import |

### 2026-10-31 · WiiGames · system notifications (#22)

| | |
| --- | --- |
| **Device** | WiiGames, Wii/GameCube USB drive, FAT32 |
| **What** | #22, system notifications when a task ends while the window is in the background |
| **Who** | Claude on petter1's Mac, the app driven in the background (computer use) |
| **When** | 2026-10-31, local merge of `main` 3935bd8e + #7 b55499ba + #20 864de29a + #22 eb16e17d (debug build, binary started from a terminal, not via the `.app`), macOS 27 |
| **How** | Hash tasks that ended while the window had no focus |
| **Why** | #22 waits for the Mac test |
| **Result** | ⚠️ The notification was sent (window unfocused, no error printed); whether a banner appeared in Notification Center wasn't seen. Tested before the later fix to #22 |
| **Follow-up** | look at Notification Center with the current branch, started as the `.app` |

### 2026-10-31 · WiiGames · the same message counted (#20)

| | |
| --- | --- |
| **Device** | WiiGames, Wii/GameCube USB drive, FAT32 |
| **What** | #20, the same message again is counted in one toast |
| **Who** | Claude on petter1's Mac, the app driven in the background (computer use) |
| **When** | 2026-10-31, local merge of `main` 3935bd8e + #7 + #20 864de29a + #22 (debug build), macOS 27 |
| **How** | Two hashes of Glacier 3 queued, both "Hash mismatch" |
| **Why** | #20 waits for the Mac test |
| **Result** | ✅ One toast "Check the hash of Glacier 3: Hash mismatch for Glacier 3" with "2×"; `log.txt` has both lines |
| **Follow-up** | none |

### 2026-10-31 · WiiGames · stay awake while a task runs (#7)

| | |
| --- | --- |
| **Device** | WiiGames, Wii/GameCube USB drive, FAT32 |
| **What** | #7, the computer is kept awake while data moves |
| **Who** | Claude on petter1's Mac, the app driven in the background (computer use) |
| **When** | 2026-10-31, local merge of `main` 3935bd8e + #7 b55499ba + #20 + #22 (debug build), macOS 27 |
| **How** | Hash of Xenoblade Chronicles; `pgrep caffeinate` and `pmset -g assertions` before, during and after |
| **Why** | #7 waits for the Mac test |
| **Result** | ⚠️ `caffeinate -i -w <app pid>` only while the task runs (PreventUserIdleSystemSleep), none before, gone after. The long run (Mac left alone until it would sleep) not done: Amphetamine and Claude hold the same assertion on this Mac |
| **Follow-up** | long run with those apps closed |

### 2026-10-31 · WiiGames · convert, export, scrub and hash as tasks, part 1 (#10)

| | |
| --- | --- |
| **Device** | WiiGames, Wii/GameCube USB drive, FAT32 |
| **What** | #10 on `main`, item 3 of "Waiting for the Mac": the parts that need no dialog and no Wii |
| **Who** | Claude on petter1's Mac, the app driven in the background (computer use) |
| **When** | 2026-10-31, `main` 3935bd8e (debug build), macOS 27 |
| **How** | Hash tasks on WiiGames: Xenoblade Chronicles (split WBFS, 7.93 GiB), GoldenEye 007, Glacier 3; one waiting task moved earlier; the running one cancelled; pause tried |
| **Why** | #10 was merged before the test on real hardware |
| **Result** | ⚠️ Second and third hash wait; "Earlier" moves Glacier 3 before GoldenEye; cancelling the running hash → "Cancelled", the next one starts; a split WBFS is read whole. Every hash: "Hash mismatch" (WBFS dumps; see #38). Pause: the background clicks fired twice (Pause then Resume, seen in a debug log), so pause by hand is still open; while paused the row kept speed and time left (fixed in #27). Not tried: export, convert to RVZ (file dialogs), scrub (needs the Wii). A hash match was shown later the same day with Worms 3D (see the #38 entry) |
| **Follow-up** | item 3 of "Waiting for the Mac": export, convert, scrub, pause by hand |

### 2026-10-31 · Mac · own icon and screenshot

| | |
| --- | --- |
| **Device** | the Mac (macOS 27, display at 1×); the 3DS petter1 (online) and the 3DS XL Petter1 (offline), the drives WiiGames and WIIRO for the screenshot |
| **What** | M0b, own face: `.icns`, screenshot, icon on macOS; item 2 of "Waiting for the Mac" |
| **Who** | petter1, with Claude |
| **When** | 2026-10-31, `main` at 45202fc (debug build), macOS 27 |
| **How** | `just icons` (resvg 0.49, ImageMagick, `iconutil`) for the `.icns`; the debug build put into a copy of `package/macos/App` as a `.app` and started; icon looked at in the Dock, ⌘-Tab, the About page and a Finder list (16 px). Screenshot of the window on My Consoles (`screencapture -iWo`), shrunk with `oxipng -o 4` |
| **Why** | the fork still had TWBM's `.icns` and screenshot; cloud sessions have no macOS |
| **Result** | ⚠️ New icon in the Dock, ⌘-Tab, the About page and Finder. The other generated files came out the same; only `icon.ico` differed in bytes with the newer ImageMagick (256 px layer stored as BMP, same pixels), so the old one stays. ⚠️ At 16 px (Finder list on a 1× display) the four letters blur into a square; 32 px reads fine. The maintainer keeps it. The screenshot is 1263×950 (1× display) instead of about 1800×1500; accepted |
| **Follow-up** | none |

### 2026-10-31 · WiiGames · import as tasks

| | |
| --- | --- |
| **Device** | WiiGames, Wii/GameCube USB drive, FAT32; WIIRO, a read-only FAT32 disk image |
| **What** | M1a, import as tasks (#5), item 1 of "Waiting for the Mac" |
| **Who** | petter1 |
| **When** | 2026-10-31, `main` with #15 (commit 4421619), macOS 27 |
| **How** | Wii and GameCube games imported to WiiGames (Wii games split); progress on the Tasks page and in the bar on other pages; one paused and resumed, one cancelled, two discs of one GameCube game with the second cancelled, a cancelled one retried. Read-only drive: no NTFS drive at hand, so a FAT32 image mounted read-only (`hdiutil attach -readonly`), then an import to it |
| **Why** | #5 was merged before the test on real hardware |
| **Result** | ✅ Works as listed. With two discs of one game, both tasks had the same title (fixed in #15, confirmed). Read-only drive: one message "WIIRO is read-only: nothing can be written to it", no task. The NTFS wording of that message isn't tested (needs a real NTFS drive). |
| **Follow-up** | none |

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
