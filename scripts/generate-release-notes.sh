#!/bin/sh

if ! grep -Fq "## [$1]" CHANGELOG.md; then
  exit
fi

awk -v t="$1" '/^## \[/ && f {exit} $0 ~ "^## \\[" t {f=1; next} f' CHANGELOG.md

echo "## ⬇️ Recommended downloads for the majority of users\n"
echo "→ [Windows Installer](https://github.com/mq1/TinyWiiBackupManagerInstaller/releases/latest/download/TinyWiiBackupManagerInstaller.exe)\\"
echo "→ [Windws x64 Standalone](https://github.com/mq1/TinyWiiBackupManager/releases/download/$1/TinyWiiBackupManager-$1-windows-x86_64.zip)\\"
echo "→ [macOS Universal Binary](https://github.com/mq1/TinyWiiBackupManager/releases/download/$1/TinyWiiBackupManager-$1-macos-universal.dmg)\\"
echo "→ [Linux Flatpak](https://flathub.org/apps/details/it.mq1.TinyWiiBackupManager)\\"
echo "→ [Linux x86_64 AppImage](https://github.com/mq1/TinyWiiBackupManager/releases/download/$1/TinyWiiBackupManager-$1-linux-x86_64.AppImage)"
