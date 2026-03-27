#!/bin/bash
# SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
# SPDX-License-Identifier: GPL-3.0-only

set -euo pipefail

ARCH=$1

export RUSTC_BOOTSTRAP=1

case $ARCH in
arm64)
  TARGET="aarch64-apple-darwin"
  ;;
x86_64)
  TARGET="x86_64-apple-darwin"
  ;;
esac

cargo build -Z build-std=std,panic_abort --release --locked --target $TARGET
cp target/$TARGET/release/TinyWiiBackupManager .
