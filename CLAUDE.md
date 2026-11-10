# Notes for Claude

Read by every Claude Code session in this repository. The maintainer writes in German; answer in
German and address them informally ("du"), keep code, commits and docs in English.

## Before starting any work

Several sessions work on this repository, often on the same day. Before writing code, look at
what already exists, so no work is done twice:

1. Read "Current state" below and the roadmap item the work belongs to.
2. `git fetch origin` and list the remote branches (`git branch -r`) and the open pull requests
   (GitHub tools). A branch or pull request for the same roadmap item means: continue there,
   don't start anew.
3. Read the code the change touches before planning it.

## What cloud sessions can't do

- **Delete remote branches or force-push**: the cloud environment (claude.ai/code) refuses
  `git push --delete` and `git push --force-with-lease` as destructive. So: never push something
  that would need to be undone that way, and use a new commit instead of amending a pushed one.
  A branch that has to go is deleted by the maintainer on GitHub. A Claude Desktop session on the
  Mac can't do it on its own either: its auto mode refused `git push --delete` (2026-11-09).

## Before a session ends

Before a session is archived (or the work is handed over), two things hold, so every session and
every branch starts from the latest state:

1. **`CLAUDE.md` on `main` is up to date**: "Current state" below describes what this session
   changed, merged in its own small `docs/` pull request.
2. **Every open branch has the new `main`** (and with it this file): `git fetch origin`, then for
   each branch of an open pull request `git merge origin/main`, resolve conflicts, run the checks
   (see "Where things are") and push. Cloud sessions merge instead of rebasing, because a rebase
   needs a force-push, which they can't do (see above); with squash merging both give the same
   commit on `main`. A session that can force-push (on the Mac) may rebase its own branches.

## Current state

Handover between sessions; the session that changes the state updates this section. New sessions
start from `main` and only see what is there, so the update goes to `main` in its own small
`docs/` pull request (merged right away), not only in the feature branch. As of 2026-11-10:

- **CI runs again** (2026-11-10 morning, all three systems; the macOS runners were missing on
  2026-11-09 evening). Merged 2026-11-10: #45 and #47 (documentation, the comparison with
  vcinject in the roadmap), #46 (both discs of a two-disc GameCube game hashed), #36 (a game
  already queued isn't queued again; tried on the Mac), #41 (M1c "Copy all to drive"; tried on
  the Mac on disk images, item 5 in "Waiting for the Mac" for WiiGames with real discs).
- **Merged 2026-10-31 after review**: #20, #24, #7 (the long sleep test is still due), #35, #38,
  #40.
- **M2, game databases** (the next milestone; built and tested in the cloud): first pull request
  **#48** (`claude/nifty-hamilton-g6dclr`), open, CI to watch, then merge after a quick review
  (no Mac needed; a scan of the real collection on the Mac shows whether the names appear). The
  No-Intro lists of GB, GBC, GBA, NES, SNES and Mega Drive from libretro-database, compacted to
  `CRC32 \t SHA-1 \t serial \t name` in `assets/databases/no-intro/` (1.9 MB, CC BY-SA 4.0;
  `scripts/update-databases.py` refreshes them; raw.githubusercontent.com is reachable from cloud
  containers, github.com and codeload aren't), built in with `include_str!`; `src/romdb.rs`
  identifies by SHA-1, CRC32, then serial. Checksums per ROM file in `library-checksums.json`
  next to `library.json` (found again by path + size + mtime, renamed or moved files by size +
  mtime); a zip with one ROM is that ROM; the same ROM data is one entry with copies. Next for
  M2: the build and the app refreshing the lists, the link to the release's entry, Redump/GameTDB
  and the other platforms, RAR via 7-Zip, artwork.
- **M1, install on the 3DS**: pull request **#11** (`claude/m1-3ds-install-and-consoles`), open,
  waiting for the Mac (item 6 in "Waiting for the Mac" on its branch): install a CIA as a task
  (no Pause: vcinject Link waits without a timeout), start and delete titles, "Update" vcinject
  Link to the 13 the app ships (`assets/vcinject-link.cia`), consoles found every 15 seconds,
  titles in `consoles.json`. **On top of it: #49** (`claude/m1-link-qr-install`, base #11's
  branch): the first install of vcinject Link by QR code for FBI ("Install vcinject Link" in My
  Consoles, a QR button next to every CIA), served by `src/link/serve.rs` while the window is
  open; item 7 in "Waiting for the Mac". Merge #11 first, then retarget #49 to `main`.
- **M1a, unfinished tasks after a restart**: pull request **#13** (`claude/m1a-resume-tasks`),
  open; try again on the Mac after the review's changes. #11 and #13 both touch `jobs.rs` and
  both call their "Waiting for the Mac" item 6: the one merged second adds `SavedJob::Install`
  (`to_saved`, `leftovers`, `restore`; `UpdateLink`/`Title` are better dropped than resumed) and
  renumbers its item.
- **M1a+, notifications**: pull request **#22** (`claude/m1a-plus-system-notifications`), open,
  reviewed; the banner hasn't been seen on the Mac (likely reasons: notify-rust on macOS sends
  when the handle is dropped and ignores errors; mac-notification-sys falls back to
  `com.apple.Terminal` for an unregistered bundle id; the old `NSUserNotificationCenter` API).
- **vcinject Link 14 – waits for the maintainer's decision**, not started: a receive timeout and
  the main loop during installs (Pause would work again), the stream install command 19 (CIAs
  over 4 GiB), answering `ugbm?`, `commands` in info. Needs devkitPro (not in cloud containers)
  and the decision where Link is developed from now on ("Open questions" in the roadmap).
- **Next on the Mac, in this order**: #13, #22, #11 and then #49 (3DS), #41 on WiiGames with
  real discs (optional), the long sleep test for #7, item 3 of "Waiting for the Mac". Each one
  that works is merged by Claude after a quick review, without asking (see "Merging" below).
- **M2b** (#26): "Hash mismatch" explains a missing update partition (#38), both discs hashed
  (#46); nod's H0–H3 validation fails on WBFS as it is (see the roadmap).
- **Trying the app in its window on the Mac**: a debug build copied under a name with
  "portable" (e.g. `UGBM-portable`) uses `UltimateGameBackupManager-data` next to it, so a
  `config.json` there with a disk image as `mount_point` and a folder of fake discs as the only
  ROM source leaves the real library alone. Clicks and screenshots from a small Swift tool
  (`CGEvent`, `CGWindowListCopyWindowInfo`, `screencapture -l`); iced has no accessibility tree.
- **Branches to delete on GitHub** (cloud sessions can't): `claude/charming-clarke-dpz4xs`,
  `claude/wizardly-fermi-whgi2g`, `claude/m2b-hash-all-discs`, `claude/no-double-import`,
  `claude/m1c-copy-many` (all merged 2026-11-10), and this pull request's
  `claude/docs-current-state-2026-11-10` once it is merged. The local `feature/task-manager` on
  the Mac goes once #11 is merged. A clone in a cloud session is shallow (50 commits): `git
  fetch --unshallow origin` before comparing old branches with `main`.
- A ROM source or drive that doesn't answer no longer holds up the library scan (#31) or the
  settings (#28). Trying a build on macOS: BUILDING.md (#29).

## Working with the maintainer

- **Merging**: Claude merges its own pull requests without asking, as soon as CI is green, the
  code is quickly reviewed and, where the roadmap says so, the test on the Mac worked.
  Documentation-only: as soon as CI is green. UI code isn't reviewed: if the UI works, it is
  good. Details: [docs/GIT_WORKFLOW.md](docs/GIT_WORKFLOW.md#pull-requests).

- **Recommend a new session proactively** as soon as it makes sense, without being asked: when
  the conversation has grown long, when the next piece of work doesn't need what this session
  knows (a new milestone, an unrelated topic), or when a fresh start is cheaper than carrying the
  context. Say so in one line, with what the new session should start with (branch, pull request,
  roadmap item).
- **Say when something takes long, or run it in the background.** A command that may run for
  more than a minute (the first `cargo build`/`clippy`/`test` in a fresh container takes several
  minutes, a clone, a large download) runs in the background, or is announced in one line before
  it starts ("Build and tests run now, about 5 minutes"). Without a word the session looks frozen.
- The maintainer tests on a Mac with real consoles and drives; cloud sessions can't. What waits
  for the Mac is listed in [docs/ROADMAP.md](docs/ROADMAP.md) under "Waiting for the Mac"; add
  to it whenever a change needs a real-hardware test.

## Where things are

- Plan and milestones: [docs/ROADMAP.md](docs/ROADMAP.md); tests on real hardware:
  [docs/TESTLOG.md](docs/TESTLOG.md).
- What [vcinject](https://github.com/petter1/vcinject) does and this app doesn't yet (compared
  2026-11-10, both `main`s and every branch): in the roadmap, marked "From vcinject" in M1, M2,
  M2b, M4, M5, M5b, M6 and M9. New in the plan since then: the first install of vcinject Link
  by QR code (M1).
- Branches, commits, pull requests: [docs/GIT_WORKFLOW.md](docs/GIT_WORKFLOW.md). Checks before
  pushing, as CI runs them: `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D
  warnings`, `cargo test --locked` (needs Rust 1.98 or newer: `rustup toolchain install stable`).
- AI rules: [AI_POLICY.md](AI_POLICY.md).
