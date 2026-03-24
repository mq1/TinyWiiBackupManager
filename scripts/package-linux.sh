#!/bin/bash
# SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
# SPDX-License-Identifier: GPL-3.0-only

VERSION_NAME=$1
export VERSION="${VERSION_NAME#v}"

ARCH_NAME=$2

case "$ARCH_NAME" in
x86_64*)
  export ARCH=x86_64
  ;;
x86)
  export ARCH=i686
  ;;
arm64)
  export ARCH=aarch64
  ;;
armhf)
  export ARCH=armhf
  ;;
esac

# Setup appdir
cp -r package/linux/AppDir TinyWiiBackupManager.AppDir
install -Dm0755 TinyWiiBackupManager TinyWiiBackupManager.AppDir/usr/bin/TinyWiiBackupManager

# Download appimagetool
wget "https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-$ARCH.AppImage" -O appimagetool
chmod +x appimagetool

mkdir -p dist
cd dist

../appimagetool \
  -u "gh-releases-zsync|mq1|TinyWiiBackupManager|latest|*$ARCH_NAME.AppImage.zsync" \
  ../TinyWiiBackupManager.AppDir \
  "TinyWiiBackupManager-${VERSION_NAME}-linux-$ARCH_NAME.AppImage"
