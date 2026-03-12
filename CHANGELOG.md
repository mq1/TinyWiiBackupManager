# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- 💪 linux-musl builds (slightly slower, extended compatibility)

### Changed

- ⚡️ Reverted to using linker-plugin-lto on linux (all targets). This bumps the min required glibc version to 2.35 (this won't affect most users), rvz performance should improve. In the rare case this doesn't work for you, I'm now providing linux-musl builds (slightly slower, extended compatibility).
- 🛠️ Added a [justfile](https://github.com/mq1/TinyWiiBackupManager/blob/main/justfile) with build scripts for anyone wanting to reproduce the builds by themselves

## [v5.1.24] - 2026-03-09

### Fixed

- 💀 Occasional deadlocks when showing dialogs on windows-legacy

## [v5.1.23] - 2026-03-07

### Changed

- ⚡️ RVZ compression/decompression should now be slightly faster on windows and windows-legacy

## [v5.1.22] - 2026-03-04

### Fixed

- 🐛 Archive Game to PC Option doesn't work (#574)

## [v5.1.21] - 2026-03-03

### Fixed

- 🐛 OS error 123 (#573)

## [v5.1.20] - 2026-03-02

### Changed

- ⚡️ Write files in 32kb chunks (potentially improving write performance on hdds, lmk if you notice any difference)
- ♻️ Code base refactoring/cleanup

## [v5.1.19] - 2026-02-27

### Fixed

- ✂️ Split writing not working correctly sometimes

## [v5.1.18] - 2026-02-26

### Fixed

- 🪟 windows x86_64-v2 didn't work on systems without avx instructions.

## [v5.1.17] - 2026-02-26

### Fixed

- 🪟 windows-legacy dialog fixes. Other platforms can easily skip this update

## [v5.1.16] - 2026-02-26

### Fixed

- 🪟 Windows (and possibly macOS) dialogs should be more reliable. windows-legacy can still be a bit less stable, I'm still trying to figure this out.

## [v5.1.15] - 2026-02-25

### Fixed

- 🐛 "Show Wii Games" and "Show GameCube Games" toggle filters don't do anything on Windows (#566)
- 🐛 Pop-up windows do not appear (#567)

## [v5.1.14] - 2026-02-24

### Fixed

- 🪟 Windows dialogs sometimes freezing the UI (#564)

## [v5.1.13] - 2026-02-22

### Changed

- 📦 macOS binaries are now only packaged as DMGs to avoid cluttering the artifact list. They should also be a little smaller

### Fixed

- 🐧 Linux builds now trigger the cpu renderer more easily to avoid rare crashes (#561)

## [v5.1.12] - 2026-02-22

### Fixed

- 🐧 Linux builds now use vulkan on newer gpus and opengl on older gpus (#546)

## [v5.1.11] - 2026-02-20

### Added

- 🐧 Linux builds now run on glibc 2.17+ (thanks to cargo-zigbuild)

### Changed

- 📦 AppImages now don't depend on zenity or kdialog (re-added custom message box only for linux builds)

### Fixed

- 🐛 Linux AppImage: stops after parsing title/titleid (#555)

## [v5.1.10] - 2026-02-19

### Added

- 🪨 Linux armv7 build (#551)
- 🔧 Support for modded wii games >= 8gb (#501)

### Changed

- 🪶 AppImage is small again as gtk3 isn't bundled anymore (either zenity or kdialog required)

### Fixed

- 📦 Correct tarball owner and group (linux dist)
- 💥 Fix crash on linux + buggy vulkan drivers (#546)
- 🐛 Thread panicked when adding a game (#553)

## [v5.1.9] - 2026-02-18

### Added

- 🪟 Experimental Windows XP and Vista support! (windows-legacy build)

### Changed

- 💬 Now using my ad-hoc developed library [blocking-dialog](https://github.com/mq1/blocking-dialog-rs) instead of rfd or native-dialog to show system dialogs / file pickers. This was needed to ensure windows xp compatibility
- 📦 AppImage doesn't depend on zenity or xdg-desktop-portal anymore. Consequently, the bundle size is bigger (flatpak and tarball not affected)

### Fixed

- 👻 Hidden file skipping (#540)

## [v5.1.8] - 2026-02-14

### Fixed

- 🪟 windows-legacy build now works on windows 7 without VxKex! (#522)
- ⛱️ Remove shadows for now to avoid glitches when cpu rendering

## [v5.1.7] - 2026-02-12

### Fixed

- ✏️ Typo in gc disc2.iso file name (#536)

## [v5.1.6] - 2026-02-12

### Changed

- 🔧 Revert to msvc on windows and gcc on linux to ensure better consistency and compatibility. More conservative compiler versions. Builds are also easier to reproduce. flatpak builds are more stable easier to mantain (as we can't pin the llvm version).

### Fixed

- 🪶 Smaller windows and windows-legacy builds
- ⛱️ Disabled shadows on windows-legacy builds, fixes graphical glitches on win7

## [v5.1.5] - 2026-02-11

### Changed

- 📦 win7 builds are now called "legacy" builds

### Fixed

- 💻 vcruntime is now correclty statically linked on windows
- 🪟 using vs2022 instead of vs2026 (may improve compatibility with win < 11)

## [v5.1.4] - 2026-02-10

### Fixed

- 💬 Message dialog not closing when installing OSC apps

## [v5.1.3] - 2026-02-10

### Changed

- 💬 Custom (faster and more reliable) message dialog implementation
- 💬 Now when you add a lot of games, the confirmation dialog lists them all

### Fixed

- 💬 Message dialog not showing on linux-AppImage + kde

## [v5.1.2] - 2026-02-08

### Fixed

- 🔄 Now using rfd as file/message dialog library, might improve reliability on linux/windows
- 💄 Wrong container rounding on linux (barely noticeable)
- 👤 Potentially fix some permission issues on windows (pt.2)

## [v5.1.1] - 2026-02-07

### Changed

- 💄 title bar is now the same color as the side panel on Windows 11 and macOS
- 💄 title bar / nav bar ui is more consistent with the rest of twbm (Windows, macOS)
- ⚡️ Revert to vulkan for linux builds
- ⚡️ Faster path normalization

### Fixed

- 💄 fix theme detection on windows
- ❌ Wrong file formats when archiving games
- 👤 Potentially fix some permission issues on windows

## [v5.1.0] - 2026-02-06

### Added

- 💽 Add a "Drive info" card in Toolbox, showing the filesystem and cluster size (and useful tips)

### Changed

- 📝 Display "No drive selected" on the title bar instead of an empty string
- ⚡️ Reduce windows 10+ binary size

### Fixed

- 🖼️ KDE Plasma now correctly shows the app icon in title bar / overview
- ⚡️ Windows drive probing should be faster and more reliable
- 🐛 Revert to using opengl on linux to improve compatibility (<https://github.com/khcrysalis/Impactor/issues/103>)
- 💽 Adding games recursively that are ZIP archived unzips file in a directory (#518, #468)
- 🔧 Certain JD Mods Appearing in GameCube Section (#520)

## [v5.0.2] - 2026-02-01

### Fixed

- 💻 TWBM not starting on macOS < 12 (#504)
- 🐛 Archive game dialog not showing different formats

## [v5.0.1] - 2026-01-31

### Added

- 🖥️ Cpu rendering on linux (used as a fallback) (#499)

### Changed

- ⚡️ The linux build now uses vulkan (instead of opengl)

### Fixed

- 🐛 TWBM not launching on Linux with hybrid graphics (#499)
- ⬆️ TWBM_DISABLE_UPDATES=1 now works again
- 💾 More aggressive fat filesystem detection (used to trigger splitting) (#501)

## [v5.0.0] - 2026-01-30

### Added

- ⬇️ [TinyWiiBackupManagerInstaller](https://github.com/mq1/TinyWiiBackupManagerInstaller), an installer for windows that automatically picks the latest release of TinyWiiBackupManager, and the right asset (optimized for your CPU)
- ⚡️ x86_64-v2 optimized builds (linux-AppImage and windows); this is detected by TinyWiiBackupManagerInstaller
- 🔎 Fuzzy game / homebrew apps search
- ⏳ Game transfer queue management (#476)
- 📥 Drag a game from your file explorer into twbm to add it
- 📥 Drag an app from your file explorer into twbm to add it
- 💿 Archive discs to any format supported by nod

### Changed

- 🧊 Port the UI to the [Iced](https://github.com/iced-rs/iced) framework (lower cpu footprint)
- 🪶 Reduce app size on Windows and macOS
- 🧰 Move Wiiload and nod-gui utilities into an unified "Toolbox" page
- 💄 Switch from [phosphor icons](https://phosphoricons.com/) to [lucide icons](https://lucide.dev/)
- 🖥️ Use software rendering on windows 7
- 🧵 Use a thread-pool to execute tasks concurrently
- 🗜️ Compress .dol and .elf files before sending them via wiiload
- 📝 titles.txt are embedded again in the executable; compression is applied, and deserialization is faster
- 👾 Downloading cheats is now more reliable for the geckocodes.org and gamehacking.org sources

### Removed

- 🍎 macOS min supported version has changed (10.12 → 10.13), following WGPU recommendations
- 🎨 Accent color selection (might be re-added later on)

### Fixed

- 🖼️ Taskbar/window icon on linux wayland is now correctly displayed
- 🗜️ Issues #492 and #494 have been fixed (large zipped games not being converted)
- 📂 Issue #495 has been fixed (resident evil 4 wrong folder name)
- ✅ More reliable fat32 checking (used to trigger .wbfs splitting)

## [v4.9.24] - 2026-01-21

### Fixed

- 🖼️ TinyWiiBackupManager icon now shows up again on the windows exe

[Unreleased]: https://github.com/mq1/TinyWiiBackupManager/compare/v5.1.24...HEAD
[v5.1.24]: https://github.com/mq1/TinyWiiBackupManager/compare/v5.1.23...v5.1.24
[v5.1.23]: https://github.com/mq1/TinyWiiBackupManager/compare/v5.1.22...v5.1.23
[v5.1.22]: https://github.com/mq1/TinyWiiBackupManager/compare/v5.1.21...v5.1.22
[v5.1.21]: https://github.com/mq1/TinyWiiBackupManager/compare/v5.1.20...v5.1.21
[v5.1.20]: https://github.com/mq1/TinyWiiBackupManager/compare/v5.1.19...v5.1.20
[v5.1.19]: https://github.com/mq1/TinyWiiBackupManager/compare/v5.1.18...v5.1.19
[v5.1.18]: https://github.com/mq1/TinyWiiBackupManager/compare/v5.1.17...v5.1.18
[v5.1.17]: https://github.com/mq1/TinyWiiBackupManager/compare/v5.1.16...v5.1.17
[v5.1.16]: https://github.com/mq1/TinyWiiBackupManager/compare/v5.1.15...v5.1.16
[v5.1.15]: https://github.com/mq1/TinyWiiBackupManager/compare/v5.1.14...v5.1.15
[v5.1.14]: https://github.com/mq1/TinyWiiBackupManager/compare/v5.1.13...v5.1.14
[v5.1.13]: https://github.com/mq1/TinyWiiBackupManager/compare/v5.1.12...v5.1.13
[v5.1.12]: https://github.com/mq1/TinyWiiBackupManager/compare/v5.1.11...v5.1.12
[v5.1.11]: https://github.com/mq1/TinyWiiBackupManager/compare/v5.1.10...v5.1.11
[v5.1.10]: https://github.com/mq1/TinyWiiBackupManager/compare/v5.1.9...v5.1.10
[v5.1.9]: https://github.com/mq1/TinyWiiBackupManager/compare/v5.1.8...v5.1.9
[v5.1.8]: https://github.com/mq1/TinyWiiBackupManager/compare/v5.1.7...v5.1.8
[v5.1.7]: https://github.com/mq1/TinyWiiBackupManager/compare/v5.1.6...v5.1.7
[v5.1.6]: https://github.com/mq1/TinyWiiBackupManager/compare/v5.1.5...v5.1.6
[v5.1.5]: https://github.com/mq1/TinyWiiBackupManager/compare/v5.1.4...v5.1.5
[v5.1.4]: https://github.com/mq1/TinyWiiBackupManager/compare/v5.1.3...v5.1.4
[v5.1.3]: https://github.com/mq1/TinyWiiBackupManager/compare/v5.1.2...v5.1.3
[v5.1.2]: https://github.com/mq1/TinyWiiBackupManager/compare/v5.1.1...v5.1.2
[v5.1.1]: https://github.com/mq1/TinyWiiBackupManager/compare/v5.1.0...v5.1.1
[v5.1.0]: https://github.com/mq1/TinyWiiBackupManager/compare/v5.0.2...v5.1.0
[v5.0.2]: https://github.com/mq1/TinyWiiBackupManager/compare/v5.0.1...v5.0.2
[v5.0.1]: https://github.com/mq1/TinyWiiBackupManager/compare/v5.0.0...v5.0.1
[v5.0.0]: https://github.com/mq1/TinyWiiBackupManager/compare/v4.9.24...v5.0.0
[v4.9.24]: https://github.com/mq1/TinyWiiBackupManager/compare/v4.9.23...v4.9.24
