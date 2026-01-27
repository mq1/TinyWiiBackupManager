#!/bin/sh

if grep -q "[$1]" CHANGELOG.md; then
  exit
fi

awk -v t="$1" '/^## \[/ && f {exit} $0 ~ "^## \\[" t {f=1; next} f' CHANGELOG.md

echo "&rarr; [Windows Installer] (https://github.com/mq1/TinyWiiBackupManagerInstaller/releases/latest/download/TinyWiiBackupManagerInstaller.exe)\\"
echo "&rarr; [macOS universal binary](https://github.com/mq1/TinyWiiBackupManager/releases/download/$1/TinyWiiBackupManager-$1-macos-universal.dmg)\\"
echo "&rarr; [Linux Flatpak](https://flathub.org/apps/details/it.mq1.TinyWiiBackupManager)"
