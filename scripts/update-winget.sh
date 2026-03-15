#!/bin/bash
# SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
# SPDX-License-Identifier: GPL-3.0-only

VERSION_NAME=$1
VERSION="${VERSION_NAME#v}"

komac update mq1.TinyWiiBackupManager \
  --version "$VERSION" \
  --urls \
  "https://github.com/mq1/TinyWiiBackupManager/releases/download/${VERSION_NAME}/TinyWiiBackupManager-${VERSION_NAME}-windows-x86_64.zip" \
  "https://github.com/mq1/TinyWiiBackupManager/releases/download/${VERSION_NAME}/TinyWiiBackupManager-${VERSION_NAME}-windows-arm64.zip" \
  "https://github.com/mq1/TinyWiiBackupManager/releases/download/${VERSION_NAME}/TinyWiiBackupManager-${VERSION_NAME}-windows-x86.zip" \
  --submit
