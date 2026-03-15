#!/bin/bash
# SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
# SPDX-License-Identifier: GPL-3.0-only

VERSION_NAME=$1
ARCH_NAME=$2
export ARCH=$3
export VERSION="${VERSION_NAME#v}"

# Download appimagetool
wget -q "https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-${ARCH}.AppImage" -O appimagetool
chmod +x appimagetool

cp -r package/linux/AppDir TinyWiiBackupManager.AppDir
install -Dm0755 TinyWiiBackupManager TinyWiiBackupManager.AppDir/usr/bin/TinyWiiBackupManager

mkdir -p dist
./appimagetool \
  -u "gh-releases-zsync|mq1|TinyWiiBackupManager|latest|*$ARCH_NAME.AppImage.zsync" \
  TinyWiiBackupManager.AppDir \
  "dist/TinyWiiBackupManager-${VERSION_NAME}-linux-$ARCH_NAME.AppImage"
cp *.zsync dist/
