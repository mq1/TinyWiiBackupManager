<p align="center">
    <img alt="logo" width="128" src="logo@2x.png">
    <h1 align="center">TinyWiiBackupManager</h1>
    <img alt="screenshot" src="screenshot.png">
</p>

🔧 A **simple** WBFS manager written in Rust

## ✨ Downloading

### Prebuilt executable (recommended)

Just grab the [latest release](https://github.com/mq1/TinyWiiBackupManager/releases/latest) for your platform

## MacOS

The app is not signed, you need to allow it with:
```sh
sudo xattr -rd com.apple.quarantine /Applications/TinyWiiBackupManager.app
```

### From source

```sh
git clone https://github.com/mq1/TinyWiiBackupManager.git
cd TinyWiiBackupManager
cargo build --release
```

The executable will be located at target/release/tiny-wii-backup-manager

## 💾 Setting up the drive

The supported configuration is a MBR Drive with a single FAT32 partition.

You must create a "wbfs" folder in the drive root.

---

❤️ Using [GameTDB](https://www.gametdb.com/)

🤖 Logo generated with Microsoft Copilot
