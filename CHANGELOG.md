# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- 💻 Fix twbm not starting on macOS < 12

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

[Unreleased]: https://github.com/mq1/TinyWiiBackupManager/compare/v5.0.0...HEAD
[v5.0.1]: https://github.com/mq1/TinyWiiBackupManager/compare/v5.0.0...v5.0.1
[v5.0.0]: https://github.com/mq1/TinyWiiBackupManager/compare/v4.9.24...v5.0.0
[v4.9.24]: https://github.com/mq1/TinyWiiBackupManager/compare/v4.9.23...v4.9.24
