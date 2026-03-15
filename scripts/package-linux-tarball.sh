#!/bin/bash
# SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
# SPDX-License-Identifier: GPL-3.0-only

VERSION_NAME=$1
ARCH=$2

mkdir -p dist
tar \
  -I 'gzip -9' \
  --owner=0 \
  --group=0 \
  --mode=0755 \
  -cvf "dist/TinyWiiBackupManager-${VERSION_NAME}-linux-${ARCH}.tar.gz" \
  TinyWiiBackupManager
