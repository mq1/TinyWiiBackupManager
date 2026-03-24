#!/bin/bash
# SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
# SPDX-License-Identifier: GPL-3.0-only

VERSION_NAME=$1
ARCH_NAME=$2

export VERSION="${VERSION_NAME#v}"

if [ "$ARCH_NAME" == "x86_64-v3" ]; then
  export ARCH=x86_64
elif [ "$ARCH_NAME" == "x86_64-v2" ]; then
  export ARCH=x86_64
elif [ "$ARCH_NAME" == "x86_64" ]; then
  export ARCH=x86_64
elif [ "$ARCH_NAME" == "x86" ]; then
  export ARCH=i686
elif [ "$ARCH_NAME" == "arm64" ]; then
  export ARCH=aarch64
elif [ "$ARCH_NAME" == "armhf" ]; then
  export ARCH=armhf
fi

cp -r package/linux/AppDir TinyWiiBackupManager.AppDir
install -Dm0755 TinyWiiBackupManager TinyWiiBackupManager.AppDir/usr/bin/TinyWiiBackupManager

mkdir -p dist
"/usr/bin/appimagetool-$ARCH" \
  -u "gh-releases-zsync|mq1|TinyWiiBackupManager|latest|*$ARCH_NAME.AppImage.zsync" \
  TinyWiiBackupManager.AppDir \
  "dist/TinyWiiBackupManager-${VERSION_NAME}-linux-$ARCH_NAME.AppImage"
cp *.zsync dist/
