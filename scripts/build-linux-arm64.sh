#!/bin/bash
# SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
# SPDX-License-Identifier: GPL-3.0-only

export RUSTC_BOOTSTRAP=1

export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C linker-plugin-lto -C link-arg=-fuse-ld=lld-21 -C link-arg=--target=aarch64-unknown-linux-gnu"
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER="clang-21"
export CC_aarch64_unknown_linux_gnu="clang-21"
export AR_aarch64_unknown_linux_gnu="llvm-ar-21"
export CFLAGS_aarch64_unknown_linux_gnu="-O3 -flto"

cargo build -Z build-std=std,panic_abort --release --locked --target aarch64-unknown-linux-gnu
cp target/aarch64-unknown-linux-gnu/release/TinyWiiBackupManager .
