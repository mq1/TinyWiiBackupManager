#!/bin/bash

if ! grep -Fq "## [$1]" CHANGELOG.md; then
  exit
fi

awk -v t="$1" '/^## \[/ && f {exit} $0 ~ "^## \\[" t {f=1; next} f' CHANGELOG.md

echo "## :arrow_down: Recommended downloads for the majority of users"
echo ""
echo "&rarr; [Windows Installer](https://github.com/mq1/TinyWiiBackupManagerInstaller/releases/latest/download/TinyWiiBackupManagerInstaller.exe)\\"
echo "&rarr; [Windows x64 Standalone](https://github.com/mq1/TinyWiiBackupManager/releases/download/$1/TinyWiiBackupManager-$1-windows-x86_64.zip)\\"
echo "&rarr; [macOS Universal Binary](https://github.com/mq1/TinyWiiBackupManager/releases/download/$1/TinyWiiBackupManager-$1-macos-universal.dmg)\\"
echo "&rarr; [Linux Flatpak](https://flathub.org/apps/details/it.mq1.TinyWiiBackupManager)\\"
echo "&rarr; [Linux x86_64 AppImage](https://github.com/mq1/TinyWiiBackupManager/releases/download/$1/TinyWiiBackupManager-$1-linux-x86_64.AppImage)"

echo ""
echo '> [!WARNING]'
echo '> windows7 builds are currently broken on windows < 8'
echo '> Additional help is needed to sort this out #522'
