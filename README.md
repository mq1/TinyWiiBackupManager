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

<table>
  <tr>
    <td width="9999px"><strong>:window: Windows</strong></td>
  </tr>
  <tr>
    <td>
      :arrow_right: <a href="https://github.com/mq1/TinyWiiBackupManagerInstaller/releases/latest/download/TinyWiiBackupManagerInstaller.exe">Download installer</a> (recommended for most users, automatic x86_64-vX detection)
      <br>
      :arrow_right: <a href="https://github.com/mq1/TinyWiiBackupManager/releases/latest">Download standalone binary</a>
      <br>
      <br>
      :warning: Windows < 10 users should use the installer, or download the <code>windows-legacy</code> standalone binary.
      <br>
      <br>
      :ice_cream: scoop:
      <br>
      <code>scoop bucket add TinyWiiBackupManager https://github.com/mq1/TinyWiiBackupManager</code>
      <br>
      <code>scoop install TinyWiiBackupManager</code>
      <br>
      <br>
      :package: winget:
      <br>
      <code>winget install -e --id mq1.TinyWiiBackupManager</code>
    </td>
  </tr>
</table>

<table>
  <tr>
    <td width="9999px"><strong>:apple: macOS</strong></td>
  </tr>
  <tr>
    <td>
      :arrow_right: <a href="https://github.com/mq1/TinyWiiBackupManager/releases/latest">Download latest DMG</a> (Universal Binary recommended for most users)
      <br>
      <br>
      :warning: The app is not notarized, you must allow it manually after installing by running this command in Terminal:
      <br>
      <code>xattr -rd com.apple.quarantine /Applications/TinyWiiBackupManager.app</code>
    </td>
  </tr>
</table>

<table>
  <tr>
    <td width="9999px"><strong>:penguin: Linux</strong></td>
  </tr>
  <tr>
    <td>
      :arrow_right: <a href="https://flathub.org/apps/it.mq1.TinyWiiBackupManager">Download on Flathub</a> (recommended for most users)
      <br>
      :arrow_right: <a href="https://github.com/mq1/TinyWiiBackupManager/releases/latest">Download latest AppImage/tarball</a>
      <br>
      <br>
      :zap: You can check if your system supports the x86_64-vX optimized binaries by running:
      <br>
      <code>ld.so --help | grep x86-64-v</code>
    </td>
  </tr>
</table>

<br>

## :snowflake: Nix / NixOS (flake)

You can build and run TinyWiiBackupManager directly with Nix flakes.

From a local checkout:

```bash
nix build path:.#default
nix run path:.#default
```

Install to your user profile:

```bash
nix profile install github:mq1/TinyWiiBackupManager
```

Open the development shell:

```bash
nix develop path:.#default
```

> [!NOTE]
> Make sure flakes are enabled in your Nix configuration (`experimental-features = nix-command flakes`).

<br>

## :page_facing_up: Additional Info

For useful tips, check out the [Wiki](https://github.com/mq1/TinyWiiBackupManager/wiki)

<br>
<br>

<p align="center"> Made with :white_heart::pink_heart::light_blue_heart::brown_heart::black_heart::heart::orange_heart::yellow_heart::green_heart::blue_heart::purple_heart: for the Wii homebrew community </p>
