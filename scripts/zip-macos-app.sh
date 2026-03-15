#!/bin/bash
# SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
# SPDX-License-Identifier: GPL-3.0-only

VERSION_NAME=$1
ARCH=$2

mkdir -p out
ditto -c -k \
  --sequesterRsrc \
  --keepParent \
  --zlibCompressionLevel 9 \
  TinyWiiBackupManager.app \
  "out/TinyWiiBackupManager-${VERSION_NAME}-macos-${ARCH}.zip"
