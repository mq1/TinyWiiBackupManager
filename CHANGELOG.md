# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- [TinyWiiBackupManagerInstaller](https://github.com/mq1/TinyWiiBackupManagerInstaller), an installer that automatically picks the latest release of TinyWiiBackupManager, and the right asset (optimized for your CPU)
- x86_64-v2 optimized builds (linux-AppImage and windows); this is detected by TinyWiiBackupManagerInstaller
- Fuzzy game / homebrew apps search
- Game transfer queue management

### Changed

- Port the UI to the [Iced](https://github.com/iced-rs/iced) framework
- Move Wiiload and nod-gui utilities into an unified "Toolbox" page
- Switch from [phosphor icons](https://phosphoricons.com/) to [lucide icons](https://lucide.dev/)
- Use software rendering on windows 7
- Use a thread-pool to execute tasks concurrently

### Removed

- macOS min supported version has changed (10.12 &rarr; 10.13), following WGPU recommendations
- Accent color selection (might be re-added later on)

## [v4.9.24] - 2026-01-21

### Fixed

- TinyWiiBackupManager icon now shows up again on the windows exe

