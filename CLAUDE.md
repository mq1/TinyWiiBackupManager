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
`docs/` pull request (merged right away), not only in the feature branch. As of 2026-11-09
(evening):

- **CI doesn't run (2026-11-09, since about 16:00)**: the macOS job got no runner and was
  cancelled after 15–20 minutes on every pull request and on `main` (#44). Not every time (times
  in UTC): #11's run of 17:42 passed on all three systems; #45's run of 17:52 passed on Linux
  and Windows, its macOS job still queued at 18:15; #36's run of 18:12 had no Linux and no macOS
  job at all (only Format and Windows, both green). Most likely the GitHub Actions minutes or
  spending limit is used up (macOS minutes count tenfold on private repositories). The
  maintainer checks GitHub → Settings → Billing → Actions. Until CI is green again, nothing is
  merged, not even documentation; the checks run locally (all green for #11, #45 and #46). If
  it doesn't come back soon, the maintainer decides: macOS out of the matrix for a while, or
  only on `main`.
- **Not merged yet: this "Current state" (#45)**. The next session merges #45 once CI is green,
  then merges `main` into every open branch (see "Before a session ends").
- **Merged 2026-10-31 after review** (all tried on the Mac before): #20 (the same message
  counted, "245×"), #24 (Messages page; its search not tried on the Mac), #7 (stay awake while
  `tasks.is_busy()`; the long sleep test is still due), #35 (a drive that doesn't answer stays in
  the list as "Doesn't answer"; since the review, a drive whose earlier read still hangs isn't read
  again), #38 (M2b first step: "Hash mismatch" says when the update partition is missing), #40
  (test log of 2026-10-31).
- **M1a, task manager**: core (#4), import (#5, #15), convert/export/scrub/hash (#10, #27) merged.
  Export, convert and scrub still need the maintainer (item 3 in "Waiting for the Mac"); pause by
  hand works (2026-11-09). Unfinished tasks after a restart: pull request **#13**
  (`claude/m1a-resume-tasks`), open. Worked on the Mac, but the review changed a lot that touches
  files on the drive (an import checks again when it starts that its disc isn't there yet; the
  cleanup keeps the files when the source was removed; an unrestorable task stays listed; a damaged
  `tasks.json` entry is skipped with a warning): try it again on the Mac before merging. #11 and #13
  both touch `jobs.rs`: the one merged second adds `SavedJob::Install` (`to_saved`, `leftovers`,
  `restore`) and renumbers its "Waiting for the Mac" item 5.
- **M1, install on the 3DS**: pull request **#11** (`claude/m1-3ds-install-and-consoles`), open,
  all three M1 items built, reviewed by the session of 2026-11-09 evening, waiting for the Mac
  (item 5 in "Waiting for the Mac"). Install a CIA as a task; cancel is checked before each chunk
  (a cancel after the last one came too late, the 3DS finished the install), and vcinject Link
  calls `AM_CancelCIAInstall` when the connection closes halfway; **no Pause for installs**
  (vcinject Link waits without a timeout and without its main loop, the 3DS looked hung). Start
  and delete titles from the title list (`Job::Title`, Delete asks first; not for vcinject Link
  itself, game card titles only start). "Update" for an older vcinject Link installs the
  vcinject Link 13 the app ships (`assets/vcinject-link.cia`, MIT, from vcinject commit 5a01743)
  and restarts it, as vcinject does (`Job::UpdateLink`). Consoles found every 15 seconds,
  titles remembered in `consoles.json` (written only when it changes). A waiting install keeps
  the console's address; after a new DHCP lease it fails (accepted, documented). The jobs #13
  has to save after #11: `SavedJob::Install`, and `UpdateLink`/`Title` are better dropped than
  resumed after a restart.
- **vcinject is the maintainer's too** (`petter1/vcinject`, MIT, made with Claude): fixes on the
  3DS side go there, as vcinject Link 14, then the app ships the new CIA. Ideas from reading
  `companion/source/main.c`: a receive timeout and the main loop during installs (Pause would
  work again), the stream install command 19 (CIAs over 4 GiB), answering the `ugbm?` query and
  `commands` in info. The CIA needs devkitPro, which cloud containers don't have: built on the
  Mac or in vcinject's CI. Not started; to decide with the maintainer.
- **M1a+, notifications**: system notifications, pull request **#22**
  (`claude/m1a-plus-system-notifications`), open, reviewed. The banner hasn't been seen on the Mac;
  likely reasons from the review: notify-rust sends on macOS when the handle is dropped and ignores
  errors there; mac-notification-sys falls back to `com.apple.Terminal` when the bundle id isn't
  registered with Launch Services; the old `NSUserNotificationCenter` API (newer one: notify-rust
  feature `preview-macos-un`, needs permission and a signed bundle). Still open for M1a+: see the
  roadmap.
- **Imports**: picking the same game twice before its import has run no longer queues two, and
  Retry refuses a game already being imported: pull request **#36** (`claude/no-double-import`),
  open. **Tried on the Mac 2026-11-09 and ready to merge once CI is green**: in the window on a
  FAT32 disk image (test log), plus four tests in `update.rs` that drive `AppState::update`
  (fake GameCube headers, the drive kept busy by another task so imports wait). Since then the
  queue is checked before the drive (a running import already has its file there).
- **M1c, copy many**: "Copy all to drive", pull request **#41** (`claude/m1c-copy-many`), open.
  **Tried on the Mac 2026-11-09** on FAT32 disk images (test log): free space, unreadable files
  and copying work; the part after the click is now `BulkCopy::read` + `bulk_copy_messages`,
  with tests. Merge after #36: merging `main` then conflicts in `update.rs` (both add a `mod
  tests` at the end; keep both, one renamed) and in `CHANGELOG.md` (keep both lines). Not tried:
  WiiGames with real (Wii) discs, a split over 4 GiB. Its branch is checked out in the worktree
  `.claude/worktrees/m0b-macos-9aa76d` on the Mac.
- **Trying the app in its window on the Mac** (2026-11-09): a debug build copied under a name
  with "portable" (e.g. `UGBM-portable`) uses `UltimateGameBackupManager-data` next to it, so a
  `config.json` there with a disk image as `mount_point` and a folder of fake discs as the only
  ROM source leaves the real library alone. Clicks and screenshots from a small Swift tool
  (`CGEvent`, `CGWindowListCopyWindowInfo`, `screencapture -l`); iced has no accessibility tree.
  A window that lost focus ignores the first click.
- **Next on the Mac, in this order**: #36 and #41 on WiiGames with real discs (optional, both
  tried on a disk image), #13 (again, after the review's changes), #22 (the banner), #11
  (3DS); then the long sleep test for #7 and item 3 of "Waiting for the Mac". Each
  one that works is merged by Claude after a quick review, without asking (see "Merging" below).
- **M2b, explain and repair bad dumps** (#26): first step merged (#38). nod's H0–H3 validation
  fails on WBFS as it is (see the roadmap). Both discs of a GameCube game on two discs are
  hashed: pull request **#46** (`claude/m2b-hash-all-discs`), open, checks green locally; merge
  once CI is green (a two-disc game on the Mac is in item 3 of "Waiting for the Mac", not needed
  before merging).
- **Roadmap order, decided 2026-11-09 evening** (the maintainer left it to Claude): after M1 and
  M1c comes **M2 (databases)**, then M2b, M3, M4, M5, M5b; M1b and M1d come after M5b, right before
  M6a, which needs both. The next new milestone work is M2; it can be built and tested in the
  cloud, while the Mac works through the queue below.
- A ROM source or drive that doesn't answer no longer holds up the library scan (#31) or the
  settings (#28). Trying a build on macOS: BUILDING.md (#29); every re-signed test build asks again
  for removable volumes access.
- Old branches deleted on GitHub 2026-11-09 at the maintainer's request:
  `feature/3ds-link-client`, `feature/drive-devices`, `feature/link-protocol`,
  `feature/m0-loose-ends`, `feature/rebrand` (all in `main`), `feature/task-manager` (replaced by
  #4, #5, #10 and #11), `pure-rust`, `test-lucide-icons-svg` (identical to upstream, where they
  stay). Only branches of open pull requests are left. The local `feature/task-manager` on the
  Mac has 4 more commits (early 3DS install and stay-awake work); delete it once #11 is merged.
  - A clone in a cloud session is shallow (50 commits): `git fetch --unshallow origin` before
    comparing old branches with `main`, or every one looks unrelated.

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
