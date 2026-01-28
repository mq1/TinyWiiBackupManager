#!/bin/bash

TARGET_RESOLUTIONS=("16x16" "24x24" "32x32" "48x48" "256x256")

# Linux
rm -rf package/linux/icons
for res in "${TARGET_RESOLUTIONS[@]}"; do
  mkdir -p package/linux/icons/hicolor/${res}/apps
  magick assets/TinyWiiBackupManager-1024x1024.png -resize ${res} package/linux/icons/hicolor/${res}/apps/it.mq1.TinyWiiBackupManager.png
  oxipng -sao6 package/linux/icons/hicolor/${res}/apps/it.mq1.TinyWiiBackupManager.png
done

# Windows
rm -f package/windows/TinyWiiBackupManager.ico
magick assets/TinyWiiBackupManager-1024x1024.png -resize 256x256 package/windows/TinyWiiBackupManager.ico
magick assets/TinyWiiBackupManager-1024x1024.png -resize 64x64 package/windows/TinyWiiBackupManager-64x64.png
oxipng -sao6 package/windows/TinyWiiBackupManager-64x64.png

# macOS
rm -f package/macos/TinyWiiBackupManager.icns
magick assets/TinyWiiBackupManager-1024x1024.png -resize 256x256 package/macos/TinyWiiBackupManager.icns
