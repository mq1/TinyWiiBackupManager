#!/bin/bash

TARGET_RESOLUTIONS=("16x16" "32x32" "48x48" "64x64" "128x128" "256x256" "512x512")
MAGICK_ARGS="-strip -colors 16"

# Linux
rm -rf package/linux/icons
for res in "${TARGET_RESOLUTIONS[@]}"; do
  mkdir -p package/linux/icons/hicolor/${res}/apps
  magick assets/TinyWiiBackupManager-1024x1024.png ${MAGICK_ARGS} -resize ${res} package/linux/icons/hicolor/${res}/apps/it.mq1.TinyWiiBackupManager.png
  oxipng -sao6 package/linux/icons/hicolor/${res}/apps/it.mq1.TinyWiiBackupManager.png
  cp package/linux/icons/hicolor/${res}/apps/it.mq1.TinyWiiBackupManager.png package/linux/icons/hicolor/${res}/apps/TinyWiiBackupManager.png
done

# Windows
rm -f package/windows/TinyWiiBackupManager.ico
magick assets/TinyWiiBackupManager-1024x1024.png ${MAGICK_ARGS} -resize 256x256 package/windows/TinyWiiBackupManager.ico
magick assets/TinyWiiBackupManager-1024x1024.png ${MAGICK_ARGS} -resize 64x64 package/windows/TinyWiiBackupManager-64x64.png
oxipng -sao6 package/windows/TinyWiiBackupManager-64x64.png

# macOS
rm -f package/macos/TinyWiiBackupManager.icns
rm -rf package/macos/TinyWiiBackupManager.iconset
mkdir package/macos/TinyWiiBackupManager.iconset
magick assets/TinyWiiBackupManager-1024x1024.png ${MAGICK_ARGS} -resize 16x16 package/macos/TinyWiiBackupManager.iconset/icon_16x16.png
oxipng -sao6 package/macos/TinyWiiBackupManager.iconset/icon_16x16.png
magick assets/TinyWiiBackupManager-1024x1024.png ${MAGICK_ARGS} -resize 32x32 package/macos/TinyWiiBackupManager.iconset/icon_16x16@2x.png
oxipng -sao6 package/macos/TinyWiiBackupManager.iconset/icon_16x16@2x.png
magick assets/TinyWiiBackupManager-1024x1024.png ${MAGICK_ARGS} -resize 64x64 package/macos/TinyWiiBackupManager.iconset/icon_32x32@2x.png
oxipng -sao6 package/macos/TinyWiiBackupManager.iconset/icon_32x32@2x.png
magick assets/TinyWiiBackupManager-1024x1024.png ${MAGICK_ARGS} -resize 128x128 package/macos/TinyWiiBackupManager.iconset/icon_128x128.png
oxipng -sao6 package/macos/TinyWiiBackupManager.iconset/icon_128x128.png
magick assets/TinyWiiBackupManager-1024x1024.png ${MAGICK_ARGS} -resize 256x256 package/macos/TinyWiiBackupManager.iconset/icon_128x128@2x.png
oxipng -sao6 package/macos/TinyWiiBackupManager.iconset/icon_128x128@2x.png
magick assets/TinyWiiBackupManager-1024x1024.png ${MAGICK_ARGS} -resize 256x256 package/macos/TinyWiiBackupManager.iconset/icon_256x256.png
oxipng -sao6 package/macos/TinyWiiBackupManager.iconset/icon_256x256.png
magick assets/TinyWiiBackupManager-1024x1024.png ${MAGICK_ARGS} -resize 512x512 package/macos/TinyWiiBackupManager.iconset/icon_256x256@2x.png
oxipng -sao6 package/macos/TinyWiiBackupManager.iconset/icon_256x256@2x.png
magick assets/TinyWiiBackupManager-1024x1024.png ${MAGICK_ARGS} -resize 512x512 package/macos/TinyWiiBackupManager.iconset/icon_512x512.png
oxipng -sao6 package/macos/TinyWiiBackupManager.iconset/icon_512x512.png
magick assets/TinyWiiBackupManager-1024x1024.png ${MAGICK_ARGS} -resize 1024x1024 package/macos/TinyWiiBackupManager.iconset/icon_512x512@2x.png
oxipng -sao6 package/macos/TinyWiiBackupManager.iconset/icon_512x512@2x.png
iconutil -c icns package/macos/TinyWiiBackupManager.iconset
