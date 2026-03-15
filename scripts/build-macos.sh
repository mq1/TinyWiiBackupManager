#!/bin/bash
# SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
# SPDX-License-Identifier: GPL-3.0-only

set -euo pipefail

export RUSTC_BOOTSTRAP=1

case $1 in
arm64)
  export MACOSX_DEPLOYMENT_TARGET="11.0"
  export CARGO_TARGET_AARCH64_APPLE_DARWIN_RUSTFLAGS="-C link-arg=-mmacosx-version-min=11.0"
  export CC_aarch64_apple_darwin="/opt/homebrew/opt/llvm@21/bin/clang"
  export CFLAGS_aarch64_apple_darwin="-O3"
  TARGET="aarch64-apple-darwin"
  ;;
x86_64)
  export MACOSX_DEPLOYMENT_TARGET="10.13"
  export CARGO_TARGET_X86_64_APPLE_DARWIN_RUSTFLAGS="-C link-arg=-mmacosx-version-min=10.13"
  export CC_x86_64_apple_darwin="/opt/homebrew/opt/llvm@21/bin/clang"
  export CFLAGS_x86_64_apple_darwin="-O3"
  TARGET="x86_64-apple-darwin"
  ;;
esac

cargo build -Z build-std=std,panic_abort --release --locked --target $TARGET
cp target/$TARGET/release/TinyWiiBackupManager .
