#!/bin/bash
# SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
# SPDX-License-Identifier: GPL-3.0-only

set -euo pipefail

VERSION_NAME=$1
ARCH=$2

mkdir -p TinyWiiBackupManager.app/Contents/MacOS TinyWiiBackupManager.app/Contents/Resources
cp TinyWiiBackupManager TinyWiiBackupManager.app/Contents/MacOS/TinyWiiBackupManager
cp package/macos/TinyWiiBackupManager.icns TinyWiiBackupManager.app/Contents/Resources/TinyWiiBackupManager.icns
cp package/macos/Info.plist TinyWiiBackupManager.app/Contents/Info.plist
/usr/libexec/PlistBuddy -c "Add :CFBundleShortVersionString string ${VERSION_NAME#v}" TinyWiiBackupManager.app/Contents/Info.plist

mkdir -p dist
ditto -c -k \
  --sequesterRsrc \
  --keepParent \
  --zlibCompressionLevel 9 \
  TinyWiiBackupManager.app \
  "dist/TinyWiiBackupManager-${VERSION_NAME}-macos-${ARCH}.zip"
