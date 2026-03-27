#!/bin/bash
# SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
# SPDX-License-Identifier: GPL-3.0-only

set -euo pipefail

ARCH=$1

export RUSTC_BOOTSTRAP=1

case $ARCH in
arm64)
  TARGET="aarch64-unknown-linux-gnu"
  ;;
armhf)
  export PKG_CONFIG_ALLOW_CROSS=1
  TARGET="armv7-unknown-linux-gnueabihf"
  ;;
x86)
  export PKG_CONFIG_ALLOW_CROSS=1
  TARGET="i686-unknown-linux-gnu"
  ;;
x86_64)
  TARGET="x86_64-unknown-linux-gnu"
  ;;
x86_64-v2)
  export RUSTFLAGS="-C target-cpu=x86-64-v2"
  export CFLAGS="-march=x86-64-v2"
  TARGET="x86_64-unknown-linux-gnu"
  ;;
x86_64-v3)
  export RUSTFLAGS="-C target-cpu=x86-64-v3"
  export CFLAGS="-march=x86-64-v3"
  TARGET="x86_64-unknown-linux-gnu"
  ;;
esac

cargo build -Z build-std=std,panic_abort --release --locked --target $TARGET --no-default-features
cp target/$TARGET/release/TinyWiiBackupManager .
