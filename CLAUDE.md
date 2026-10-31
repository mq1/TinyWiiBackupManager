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
  A branch that has to go is deleted by the maintainer on GitHub, or by a Claude Desktop session
  on the Mac, which can probably do it (not tried yet).

## Current state

Handover between sessions; the session that changes the state updates this section. New sessions
start from `main` and only see what is there, so the update goes to `main` in its own small
`docs/` pull request (merged right away), not only in the feature branch. As of 2026-10-31:

- **M1a, task manager** (see the roadmap):
  1. Core: merged (#4).
  2. Import as tasks: merged (#5).
  3. Convert, export, scrub and checksums as tasks, `LongOperations` removed, reordering: merged
     (#10).
     Both were merged before the test on the Mac; it is still due (see "Waiting for the Mac" in
     the roadmap).
  4. Stay awake: pull request **#7**, branch `claude/m1a-stay-awake`, open; was branched before
     #5 and needs `main` merged in.
  5. Unfinished tasks after a restart: pull request **#13**, branch `claude/m1a-resume-tasks`,
     open, waits for the Mac test. #11 and #13 each add a "Waiting for the Mac" item 5 and
     touch `jobs.rs`: the one merged second renumbers its item and adds `SavedJob::Install`.
- **M1, install on the 3DS**: pull request **#11**, branch `claude/m1-3ds-install-and-consoles`,
  open: "Install" on a CIA as a task (`Job::Install`, `Resource::Console`), consoles found in
  the background every 15 seconds (failures only logged), installed titles remembered in
  `consoles.json`. Ported from the local branch `feature/task-manager`, which predates #4 and
  has its own task manager; nothing else there is needed (its M1d roadmap is #8, staying awake
  is #7). Waits for the Mac test (item 5 in "Waiting for the Mac" on that branch) before merging.
- **M0b**: only the Mac part is left (`.icns`, screenshot).
- Branches to delete on GitHub: `feature/task-manager` once #11 is merged. Sessions on the
  maintainer's Mac can delete remote branches (done for the earlier ones on 2026-10-31).

## Working with the maintainer

- **Recommend a new session proactively** as soon as it makes sense, without being asked: when
  the conversation has grown long, when the next piece of work doesn't need what this session
  knows (a new milestone, an unrelated topic), or when a fresh start is cheaper than carrying the
  context. Say so in one line, with what the new session should start with (branch, pull request,
  roadmap item).
- The maintainer tests on a Mac with real consoles and drives; cloud sessions can't. What waits
  for the Mac is listed in [docs/ROADMAP.md](docs/ROADMAP.md) under "Waiting for the Mac"; add
  to it whenever a change needs a real-hardware test.

## Where things are

- Plan and milestones: [docs/ROADMAP.md](docs/ROADMAP.md); tests on real hardware:
  [docs/TESTLOG.md](docs/TESTLOG.md).
- Branches, commits, pull requests: [docs/GIT_WORKFLOW.md](docs/GIT_WORKFLOW.md). Checks before
  pushing, as CI runs them: `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D
  warnings`, `cargo test --locked` (needs Rust 1.98 or newer: `rustup toolchain install stable`).
- AI rules: [AI_POLICY.md](AI_POLICY.md).
