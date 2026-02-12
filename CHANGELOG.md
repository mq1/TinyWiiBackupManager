# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- 🪶 Smaller windows-legacy builds

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
- 🐛 Revert to using opengl on linux to improve compatibility (https://github.com/khcrysalis/Impactor/issues/103)
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

[Unreleased]: https://github.com/mq1/TinyWiiBackupManager/compare/v5.1.5...HEAD
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
