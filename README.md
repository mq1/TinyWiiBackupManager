<img alt="TinyWiiBackupManager Logo" width="128" src="assets/TinyWiiBackupManager-256x256.png" align="left">

### `TinyWiiBackupManager`<br><sub><sup>:star: A tiny game backup and homebrew app manager for the Wii</sup></sub>

[![release: vX.X.X](https://img.shields.io/github/v/release/mq1/TinyWiiBackupManager)](#arrow_down-downloads)
[![license: GPL-3.0](https://img.shields.io/github/license/mq1/TinyWiiBackupManager)](https://github.com/mq1/TinyWiiBackupManager/blob/main/COPYING)

<br>

> [!CAUTION]
> TinyWiiBackupManager is intended strictly for legal homebrew use and is not affiliated with or endorsed by Nintendo.
> Use of TinyWiiBackupManager for pirated or unauthorized copies of games is strictly prohibited.

<img align="center" alt="App Screenshot" src="assets/screenshot.png">

## :sparkles: Features

- **Lightweight & Fast**: Native app, -O3, LTO, x86_64-v1/v2/v3 optimized builds
- **Cross-Platform**:
  - :window: Windows XP+ | x86 (32-bit), x86_64 (64-bit), arm64 (Qualcomm Snapdragon etc.)
  - :apple: macOS 10.13+ | x86_64 (Intel), arm64 (Apple Silicon/M1+)
  - :penguin: Linux 3.2+ (glibc 2.17+) | x86 (32-bit), x86_64 (64-bit), armhf/arm64 (Raspberry PIs etc.)

#### :video_game: Game Management

- **Games view**: Manage your Wii and GameCube games
- **Format Support**: .iso, .rvz and major formats thanks to [NOD](https://github.com/encounter/nod)
- **Automatic Splitting**: .wbfs file splitting when needed
- **Partition Stripping**: Remove the update partition to save space
- **Game Archiving**: Archive games using RVZ+zstd-19
- **Integrity Checks**: Verify game data for corruption
- **GameTDB**: Fetch covers and `wiitdb.xml` from GameTDB
- **TxtCodes**: Download cheat codes from geckocodes.org (web archive), codes.rc24.xyz and gamehacking.org

#### :toolbox: Wii Homebrew Management

- **Apps view**: Manage Wii homebrew applications
- **OSC view**: Download apps from the Open Shop Channel
- **Wiiload**: Send apps directly to Wii via network

<br>

## :arrow_down: Downloads

- **:window: Windows**:\
  :arrow_right: [Download installer](https://github.com/mq1/TinyWiiBackupManagerInstaller/releases/latest/download/TinyWiiBackupManagerInstaller.exe) (recommended for most users, automatic x86_64-vX detection)\
  :arrow_right: [Download standalone binary](https://github.com/mq1/TinyWiiBackupManager/releases/latest)\
  :warning: Windows < 10 users should use the installer, or download the `windows-legacy` standalone binary.

  - :ice_cream: scoop:\
    `scoop bucket add TinyWiiBackupManager https://github.com/mq1/TinyWiiBackupManager`\
    `scoop install TinyWiiBackupManager`
  - :package: winget:\
    `winget install -e --id mq1.TinyWiiBackupManager`

<br>

- **:apple: macOS**\
  :arrow_right: [Download latest dmg/zip](https://github.com/mq1/TinyWiiBackupManager/releases/latest) (_universal.dmg_ recommended for most users)\
  :warning: The app is not notarized, you must allow it manually after installing by running this command in Terminal:\
  `xattr -rd com.apple.quarantine /Applications/TinyWiiBackupManager.app`

<br>

- **:penguin: Linux**\
  :arrow_right: [Download on Flathub](https://flathub.org/apps/it.mq1.TinyWiiBackupManager) (recommended for most users)\
  :arrow_right: [Download latest AppImage/tarball](https://github.com/mq1/TinyWiiBackupManager/releases/latest)\
  :zap: You can check if your system supports the x86_64-vX optimized binaries by running:\
  `/lib64/ld-linux-x86-64.so.2 --help | grep x86-64-v`

<br>

## :page_facing_up: Additional Info

For useful tips, check out the [Wiki](https://github.com/mq1/TinyWiiBackupManager/wiki)

<br>
<br>

<p align="center"> Made with :white_heart::pink_heart::light_blue_heart::brown_heart::black_heart::heart::orange_heart::yellow_heart::green_heart::blue_heart::purple_heart: for the Wii homebrew community </p>
