<img alt="Ultimate Game Backup Manager Logo" width="128" src="assets/UltimateGameBackupManager-256x256.png" align="left">

### `Ultimate Game Backup Manager`<br><sub><sup>:star: One app for the game backups, homebrew and consoles of the whole Nintendo family</sup></sub>

[![license: GPL-3.0](https://img.shields.io/github/license/petter1/UltimateGameBackupManager)](https://github.com/petter1/UltimateGameBackupManager/blob/main/COPYING)
[![based on: TinyWiiBackupManager](https://img.shields.io/badge/based%20on-TinyWiiBackupManager-blue)](https://github.com/mq1/TinyWiiBackupManager)

<br>

> [!IMPORTANT]
> **This is a fork of [TinyWiiBackupManager](https://github.com/mq1/TinyWiiBackupManager) by
> [Manuel Quarneti (mq1)](https://github.com/mq1).** Everything that works today – the UI,
> the Wii and GameCube game management, the [Open Shop Channel](https://oscwii.org),
> [Wiiload](https://wiibrew.org/wiki/Wiiload) – is mq1's work.
> Please don't report bugs of this fork to TinyWiiBackupManager.

> [!NOTE]
> **This fork is written with AI.** Its code, docs and commits are produced by [Claude](https://claude.ai) ([Anthropic](https://www.anthropic.com))
> together with a human who decides what gets built, tests it on real consoles and owns the result.
> See [AI_POLICY.md](AI_POLICY.md).

> [!CAUTION]
> Ultimate Game Backup Manager is intended strictly for legal homebrew use and is not affiliated
> with or endorsed by Nintendo. Use it only with games you own.

<img align="center" alt="App Screenshot" src="assets/screenshot.png">

## :scroll: How this came to be

First there was [vcinject](https://github.com/petter1/vcinject), a tool to build Virtual
Console injects for the 3DS, with a little 3DS app called *vcinject Link* to manage consoles over
Wi-Fi. It grew. It got a GUI. It wanted to do the Wii and the Wii U next.

Then its author found TinyWiiBackupManager, opened it, and fell in love with the UI on the spot.
Rather than rebuild something that pretty from scratch, the plan changed: take TWBM as the
foundation and teach it more consoles, one at a time.

*(The AI would like it on record that it smuggled this paragraph in. It was told to be honest
about who writes the code, and it takes that very seriously.)*

## :sparkles: Features

Everything TinyWiiBackupManager does:

- **Lightweight & fast**: native app written in [Rust](https://www.rust-lang.org) and [Iced](https://iced.rs), no
  [Electron](https://www.electronjs.org)
- **Platforms**: Windows XP+, macOS 10.12+, Linux ~2014+ (x86, x64, arm64)
- **Wii and GameCube games**: .iso, .rvz and more thanks to [NOD](https://github.com/encounter/nod),
  .wbfs splitting, update partition stripping, RVZ archiving, integrity checks
- **[GameTDB](https://www.gametdb.com)** covers and `wiitdb.xml`, **TxtCodes** cheat codes
- **Wii homebrew**: manage apps, download from the [Open Shop Channel](https://oscwii.org), send with
  [Wiiload](https://wiibrew.org/wiki/Wiiload)

## :jigsaw: Injects

An *inject* is an older game packed so that a newer console runs it as its own title, through
the console's Virtual Console emulator: a GBA game on the 3DS HOME Menu, an N64 game on the Wii U.

- **Planned**: 3DS – GBA, NES, SNES, Game Boy; Wii U – N64, DS, GBA, SNES, NES, GameCube, Wii.
- **Identified by content**: the app finds the ROM or disc inside an inject and identifies it by
  its checksum, so an inject is recognised as the game it contains, whatever its file is called.
- **One entry per game**: the ROM and every inject made from it are one entry in the library,
  which shows the consoles the game can be installed on ("installable on").
- **Building them**: 3DS injects first through [vcinject](https://github.com/petter1/vcinject),
  later ported to Rust; Wii U injects modelled on
  [UWUVCI](https://github.com/stuff-by-3-random-dudes/UWUVCI-AIO-WPF).
- **Nothing is shipped**: no console keys, no ROMs, no firmware. Keys a console needs (e.g. the
  Wii U common key) are entered by you, once, and stay on your computer.

## :world_map: Roadmap

Every console becomes a module with the same building blocks: games, homebrew, an injector and a
companion app that lets this app manage the console over Wi-Fi. Next up are a task manager with
progress bars, installing on the 3DS over Wi-Fi and preparing SD cards and drives; after that game
databases, injects, missing games and collections, the Wii and the Wii U, and a server version
for a NAS. The milestones, in order: [docs/ROADMAP.md](docs/ROADMAP.md).

## :arrow_down: Downloads

There are no releases of this fork yet. Until there are, build it yourself: see
[BUILDING.md](BUILDING.md). If you only need Wii and GameCube, use
[TinyWiiBackupManager](https://github.com/mq1/TinyWiiBackupManager/releases/latest) – it's the
original and it's great.

## :handshake: Contributing

How branches, commits, pull requests and releases work here: [docs/GIT_WORKFLOW.md](docs/GIT_WORKFLOW.md).
What's planned: [docs/ROADMAP.md](docs/ROADMAP.md). What was tested on real hardware:
[docs/TESTLOG.md](docs/TESTLOG.md).

## :heart: Credits

**[Manuel Quarneti (mq1)](https://github.com/mq1)** wrote TinyWiiBackupManager, and with it the
whole base of this app. Thank you.

Thanks as well to:

- the TinyWiiBackupManager contributors:
  [encounter](https://github.com/encounter), [mak448a](https://github.com/mak448a),
  [brunochanrio](https://github.com/brunochanrio), [GABO1423](https://github.com/GABO1423)
  and [everyone else](https://github.com/mq1/TinyWiiBackupManager/graphs/contributors)
- [Luke Street (encounter)](https://github.com/encounter) for [NOD](https://github.com/encounter/nod)
- [blackb0x](https://github.com/wiidev) for the feedback and advice that shaped TWBM
- [GameTDB](https://www.gametdb.com), the [Open Shop Channel](https://oscwii.org) and the
  cheat code archives TWBM uses: [GeckoArchive](https://github.com/mq1/GeckoArchive),
  [GameHacking.org](https://gamehacking.org) and [RiiConnect24](https://rc24.xyz)
- [Rust](https://www.rust-lang.org), [Iced](https://iced.rs) and
  [Lucide](https://lucide.dev) (icons), which the app is built with
- for the 3DS work coming over from vcinject: [3dbrew](https://www.3dbrew.org),
  [devkitPro](https://devkitpro.org), [Luma3DS](https://github.com/LumaTeam/Luma3DS),
  [Universal-Team](https://github.com/Universal-Team) and the rest of the
  [vcinject credits](https://github.com/petter1/vcinject#credits)
- [UWUVCI](https://github.com/stuff-by-3-random-dudes/UWUVCI-AIO-WPF), the gold standard for
  Wii U injects and an inspiration for vcinject

## :coin: Donations

This fork doesn't take donations. If you'd like to give something back, give it to the people
who built the foundation: star and support [TinyWiiBackupManager](https://github.com/mq1/TinyWiiBackupManager),
[mq1](https://github.com/mq1) and its contributors.

## :balance_scale: License

[GPL-3.0-only](COPYING), like TinyWiiBackupManager. Code taken over from
[vcinject](https://github.com/petter1/vcinject) (MIT) keeps its
copyright notice.

<br>

<p align="center"> Made with :white_heart::pink_heart::light_blue_heart::brown_heart::black_heart::heart::orange_heart::yellow_heart::green_heart::blue_heart::purple_heart: for the Nintendo homebrew community, on the shoulders of TinyWiiBackupManager </p>
