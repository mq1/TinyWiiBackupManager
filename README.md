<img alt="TinyWiiBackupManager Logo" width="128" src="assets/TinyWiiBackupManager.png" align="left">

### `TinyWiiBackupManager` <sub><sup>(A tiny game backup and homebrew app manager for the Wii)</sup></sub>

<sub>‼️ TinyWiiBackupManager is intended strictly for legal homebrew use and is not affiliated with or endorsed by
Nintendo.<br>‼️ Use of TinyWiiBackupManager for pirated or unauthorized copies of games is strictly prohibited.</sub>

<p align="center">
  <img alt="App Screenshot" src="assets/screenshot.png">
</p>

## ✨ Features

- **Lightweight & Fast**: Native DX12/Metal/OpenGL rendering, -O3, LTO, x86_64-v3 optimized builds
- **Cross-Platform**: Windows 7+ (x86, x64, arm64), macOS 10.12+ (x64, arm64), Linux (x86, x64, arm64)

#### 🎮 Game Management

- **Games view**: Manage your Wii and GameCube games
- **Format Support**: .iso, .rvz and major formats thanks to [NOD](https://github.com/encounter/nod)
- **Automatic Splitting**: .wbfs file splitting when needed
- **Partition Stripping**: Remove the update partition to save space
- **Game Archiving**: Archive games using RVZ+zstd-19
- **Integrity Checks**: Verify game data for corruption
- **GameTDB**: Fetch covers and `wiitdb.xml` from GameTDB
- **TxtCodes**: Download cheat codes from geckocodes.org (web archive), codes.rc24.xyz and gamehacking.org

#### 🛠️ Wii Homebrew Management

- **Apps view**: Manage Wii homebrew applications
- **OSC view**: Download apps from the Open Shop Channel
- **Wiiload**: Send apps directly to Wii via network

## ⬇️ Downloads

<table>
  <tr>
    <td>
        &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;
        <br>
        <img height="48px" src="https://raw.githubusercontent.com/edent/SuperTinyIcons/refs/heads/master/images/svg/windows.svg">
        <br>
        Windows
        <br>
        &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;
    </td>
    <td>
        <a href="https://github.com/mq1/TinyWiiBackupManager/releases/latest"><img alt="⬇️ Standalone/Installer" src="https://img.shields.io/github/v/release/mq1/TinyWiiBackupManager?logo=github&label=%E2%AC%87%20Standalone/Installer"></a>
        <br><br>
        ℹ️ If you don't know which asset to download, <kbd>TinyWiiBackupManager-vX.X.X-windows-x86_64.zip</kbd> should work for most users
        <br><br>
        ⚡️ If you have a recent CPU (see <a href="https://en.wikipedia.org/wiki/X86-64#Microarchitecture_levels">here</a>), you can get the slightly faster <kbd>TinyWiiBackupManager-vX.X.X-windows-x86_64-v3.zip</kbd>
    </td>
  </tr>
</table>

<table>
  <tr>
    <td>
        &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;
        <br>
        <img height="48px" src="https://raw.githubusercontent.com/edent/SuperTinyIcons/refs/heads/master/images/svg/linux.svg">
        <br>
        Linux
        <br>
        &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;
    </td>
    <td>
        <a href="https://github.com/mq1/TinyWiiBackupManager/releases/latest"><img alt="⬇️ AppImage" src="https://img.shields.io/github/v/release/mq1/TinyWiiBackupManager?logo=github&label=%E2%AC%87%20AppImage"></a> <a href="https://flathub.org/apps/it.mq1.TinyWiiBackupManager"><img alt="⬇️ Flatpak" src="https://img.shields.io/flathub/v/it.mq1.TinyWiiBackupManager?logo=flathub&label=%E2%AC%87%20Flatpak"></a>
        <br><br>
        ℹ️ If you don't know which asset to download, <kbd>TinyWiiBackupManager-vX.X.X-linux-x86_64.AppImage</kbd> should work for most users
        <br><br>
        ⚡️ If you have a recent CPU (see <a href="https://en.wikipedia.org/wiki/X86-64#Microarchitecture_levels">here</a>), you can get the slightly faster <kbd>TinyWiiBackupManager-vX.X.X-linux-x86_64-v3.AppImage</kbd>
    </td>
  </tr>
</table>

<table>
  <tr>
    <td>
        &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;
        <br>
        <img height="48px" src="https://raw.githubusercontent.com/edent/SuperTinyIcons/refs/heads/master/images/svg/apple.svg">
        <br>
        macOS
        <br>
        &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;
    </td>
    <td>
        <a href="https://github.com/mq1/TinyWiiBackupManager/releases/latest"><img alt="⬇️ DMG/ZIP" src="https://img.shields.io/github/v/release/mq1/TinyWiiBackupManager?logo=github&label=%E2%AC%87%20DMG/ZIP"></a>
        <br><br>
        ℹ️ If you don't know which asset to download, <kbd>TinyWiiBackupManager-vX.X.X-macos-arm64.dmg</kbd> should work for most users
        <br><br>
        ⚠️ The app is not notarized, you must allow it manually after installing by running this command in Terminal:
        <br>
        <code>xattr -rd com.apple.quarantine /Applications/TinyWiiBackupManager.app</code>
    </td>
  </tr>
</table>

<table>
  <tr>
    <td>
        &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;
        <br>
        &nbsp;&nbsp;&nbsp;<img height="36px" src="https://www.svgrepo.com/download/355384/windows-legacy.svg">
        <br>
        Windows 7
        <br>
        &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;
    </td>
    <td>
        <a href="https://github.com/mq1/TinyWiiBackupManager/releases/latest"><img alt="⬇️ Standalone/Installer" src="https://img.shields.io/github/v/release/mq1/TinyWiiBackupManager?logo=github&label=%E2%AC%87%20Standalone/Installer"></a>
        <br><br>
        ℹ️ If you don't know which asset to download, <kbd>TinyWiiBackupManager-vX.X.X-windows7-x86.zip</kbd> should work for most users
        <br><br>
        ⚠️ This package is untested and may not work
    </td>
  </tr>
</table>

## 📄 Additional Info

For useful tips, check out the [Wiki](https://github.com/mq1/TinyWiiBackupManager/wiki)

<br>
<br>

<p align="center"> Made with 🤍🩷🩵🤎🖤❤️🧡💛💚💙💜 for the Wii homebrew community </p>
