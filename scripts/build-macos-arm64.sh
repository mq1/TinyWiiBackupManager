#!/bin/bash
# SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
# SPDX-License-Identifier: GPL-3.0-only

export RUSTC_BOOTSTRAP=1

export MACOSX_DEPLOYMENT_TARGET="11.0"
export CARGO_TARGET_AARCH64_APPLE_DARWIN_RUSTFLAGS="-C link-arg=-mmacosx-version-min=11.0"
export CC_aarch64_apple_darwin="/opt/homebrew/opt/llvm@21/bin/clang"
export CFLAGS_aarch64_apple_darwin="-O3"

cargo build -Z build-std=std,panic_abort --release --locked --target aarch64-apple-darwin
cp target/aarch64-apple-darwin/release/TinyWiiBackupManager .
