# Git workflow

How changes get into Ultimate Game Backup Manager. The goal: `main` can always be built and
used, and its history reads like a list of what changed and why, one line per change.

## In short

1. Every change starts on a short-lived **branch** from `main`.
2. It goes into `main` through a **pull request**: CI must be green, and the maintainer reviews it.
3. The pull request is **squash-merged**: one commit per pull request on `main`, history stays
   linear, no merge commits.
4. **Releases** are tags `vX.Y.Z` on `main`; the tag builds and publishes them.

## Branches

| Branch | For | Example |
| --- | --- | --- |
| `main` | the current state; always green, never force-pushed | |
| `feat/<topic>` | a new feature or part of a milestone | `feat/m1a-task-manager` |
| `fix/<topic>` | a bug fix | `fix/readonly-drive-message` |
| `docs/<topic>` | documentation, roadmap, test log | `docs/git-workflow` |
| `chore/<topic>` | dependencies, build, CI, refactoring without a visible change | `chore/update-iced` |
| `claude/<topic>` | changes made in a Claude Code session (the session picks the name) | `claude/docs-file-mgmt-docker-…` |

- Topics are lowercase with hyphens; a milestone's number goes first when there is one
  (`m1b-prepare-sd-card`).
- A branch is for one thing. Something else found on the way gets its own branch or an issue.
- A branch lives for days, not weeks. A large milestone is split into several pull requests that
  each leave `main` usable (behind a setting or not yet linked in the UI if needed).
- Merged branches are deleted.
- Keeping a branch up to date with `main`: `git rebase origin/main` on your own branch (then
  `git push --force-with-lease`); never rebase or force-push a branch someone else works on.

## Commits

[Conventional Commits](https://www.conventionalcommits.org/), as the history already uses:

```
<type>(<scope>): <what changed, in the imperative or as a result>

<why, if it isn't obvious; what was tried; what is left>

Refs #12
Co-Authored-By: Claude … (for commits written with AI, see AI_POLICY.md)
```

- **Types**: `feat`, `fix`, `docs`, `chore`, `build`, `ci`, `refactor`, `test`, `perf`.
  (Older entries also use `doc` and `simplify`; new ones don't.)
- **Scopes** name the part of the app: `library`, `consoles`, `drives`, `link`, `n3ds`, `games`,
  `homebrew`, `osc`, `config`, `tasks`, `notifications`, `ui`, `roadmap`, `testlog`. A new part
  of the app brings a new scope.
- The subject is at most about 72 characters, lowercase, no period at the end.
- A breaking change (config format, data folder, Link protocol) gets a `!` after the scope and a
  `BREAKING CHANGE:` line in the body.

## Pull requests

- **Title** in commit format; it becomes the commit on `main` (`feat(library): …`).
- **Description** follows the [template](../.github/pull_request_template.md): what and why,
  the roadmap milestone, how it was tested, and the checklist.
- **Before merging**:
  - CI is green (format, Clippy, tests on Linux, macOS and Windows).
  - `CHANGELOG.md` has the change under `[Unreleased]` when users notice it.
  - A change tried on real hardware has an entry in the [test log](TESTLOG.md); a milestone
    counts as done only with one (see the [roadmap](ROADMAP.md)).
  - The roadmap is updated when the plan changed or a milestone is done.
- **Squash merge**, with the pull request number in the commit (GitHub adds `(#12)`). The
  `Co-Authored-By` lines of the branch's commits stay in the body.
- Pull requests opened by Claude are reviewed and merged by the maintainer, who is responsible
  for every change (see [AI_POLICY.md](../AI_POLICY.md)). Claude merges only when the maintainer
  says so for that pull request.

### When a pull request isn't needed

None: also small documentation changes go through one, so every change on `main` has a pull
request to look up. A pull request may contain only a typo fix and be merged right away.

## Releases

- Versions follow [Semantic Versioning](https://semver.org/); the fork started at `0.1.0`.
  While the version is `0.x`, a minor version (`0.2.0`) may contain breaking changes, a patch
  (`0.2.1`) only fixes.
- A release is a pull request `chore: release vX.Y.Z` that sets the version in `Cargo.toml`
  (and `Cargo.lock`), moves `[Unreleased]` in `CHANGELOG.md` to the new version with the date
  and updates `package/release-body.md`.
- After it is merged, the maintainer tags that commit on `main` and pushes the tag:
  `git tag -a vX.Y.Z -m "vX.Y.Z" && git push origin vX.Y.Z`. The release workflow builds and
  publishes it.
- A fix for a release is made on `main` and released as the next patch version; there are no
  release branches.

## TinyWiiBackupManager (upstream)

The fork is based on [TinyWiiBackupManager](https://github.com/mq1/TinyWiiBackupManager).

- Add it once as a remote: `git remote add upstream https://github.com/mq1/TinyWiiBackupManager.git`.
- Changes from there are taken one by one on a branch `chore/upstream-<topic>`, with
  `git cherry-pick -x <commit>` so that each commit names its original, and go in through a
  pull request like any other change. Upstream is not merged as a whole: the fork has been renamed
  and restructured too much for that.
- Nothing is sent upstream by an agent (see [AI_POLICY.md](../AI_POLICY.md)).

## GitHub settings for this

Set once by the maintainer under *Settings* of the repository:

- *General → Pull Requests*: allow only **squash merging**, "Default commit message: pull request
  title and description", **automatically delete head branches**.
- *Rules → Rulesets* for `main`: require a pull request, require the status checks
  `Format` and `Clippy & tests (…)` for all three systems, block force pushes and deletion. The
  maintainer may bypass the rules for emergencies.
