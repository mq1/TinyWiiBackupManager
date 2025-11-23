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

#### 🛠️ Wii Homebrew Management

- **Apps view**: Manage Wii homebrew applications
- **OSC view**: Download apps from the Open Shop Channel
- **Wiiload**: Send apps directly to Wii via network

## ⬇️ Downloads

[![Download latest release (Windows/macOS/Linux)](https://img.shields.io/github/v/release/mq1/TinyWiiBackupManager?logo=github&label=Download%20latest%20release%20(Windows/macOS/Linux))](https://github.com/mq1/TinyWiiBackupManager/releases/latest)\
[![Download on Flathub (Linux only)](https://img.shields.io/flathub/v/it.mq1.TinyWiiBackupManager?logo=flathub&label=Download%20on%20Flathub%20(Linux%20only))](https://flathub.org/apps/it.mq1.TinyWiiBackupManager)\
[![Winget: mq1.TinyWiiBackupManager](https://img.shields.io/winget/v/mq1.TinyWiiBackupManager?label=Winget:%20mq1.TinyWiiBackupManager&logo=data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABAAAAAQCAYAAAAf8/9hAAAACXBIWXMAAAsTAAALEwEAmpwYAAAAAXNSR0IArs4c6QAAAARnQU1BAACxjwv8YQUAAAHeSURBVHgBlVK9bhRBDP68bC458aOcQkSHNlWkIFEjapoU/DwEvAEIBSQKOmpKeANEoKAjJS8AEdehuwYSDhSELkpyOzvj2J6Zy16qZFYje+3Pn7+ZMSGtt4+6AzBVOMsiDB++OVhRt8yx0X9UQLAsEcCcseKfri8Ue5LH1su1gZgK51vDOy/6K6ZgZ89VOcrSmohmkBwT07hhCrIa+vHpyUACFbM3ZAhBjHxitYg5pB39GMtxHpbeu4pTECGBjYAjmfwTUpGSIseNpCp901iw3TEoiRCs3ntluf7mY8QLjjhrZjhGGXwzlYgsVUBOiMtuz858NJmgM0cRlxqFpEYIXEpwlKfnlKT3PoEA3zgEIuSjcvBTv/CiQLcqUeCF+UUsrd1FIZZTp2Khh6Ub98UuRrLQxC3Ny9C49q2id/0Wrt18gCvVbVOha3X9ORYuL6Oua/z99q71KqbAQXfwkfHn9y2M/+2ic/GqdVeS+UvLGO/tYLf/+USt1TgUoSVfLR2OsP3hmRT8MgLd6n/d3AAf/BZcHbGN1jTxDkJiy+crJn+w/XHDCve1+P1TzNWjhJtVQF9er3N72uITxakcH9Yg+e92Cn2fab6Nl0msh3GUZ+eArTBPpJt5QuTJlFE+BgBk8ZTIJlwgAAAAAElFTkSuQmCC)](#)

> **ℹ️ I don't know which asset I should download**\
> On Windows, you'll usually download `TinyWiiBackupManager-vX.X.X-windows-x86_64.zip`\
> On macOS, you'll usually download `TinyWiiBackupManager-vX.X.X-macos-arm64.dmg`\
> On Linux, you'll usually download `TinyWiiBackupManager-vX.X.X-linux-x86_64.AppImage`\
> If you have a recent CPU (see [here](https://en.wikipedia.org/wiki/X86-64#Microarchitecture_levels)), you can get the
> slightly faster x86_64-v3 binary

> **⚠️ macOS post-installation**\
> The app is not notarized, you must allow it manually after installing by running this command in Terminal:
>```sh
>xattr -rd com.apple.quarantine /Applications/TinyWiiBackupManager.app
>```

## 📄 Additional Info

For useful tips, check out the [Wiki](https://github.com/mq1/TinyWiiBackupManager/wiki)

<br>
<br>

<p align="center"> Made with 🤍🩷🩵🤎🖤❤️🧡💛💚💙💜 for the Wii homebrew community </p>
