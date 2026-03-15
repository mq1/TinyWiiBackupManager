#!/bin/bash
# SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
# SPDX-License-Identifier: GPL-3.0-only

export RUSTC_BOOTSTRAP=1

export CARGO_TARGET_I686_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C linker-plugin-lto -C link-arg=-fuse-ld=lld-21 -C link-arg=--target=i686-unknown-linux-gnu"
export CARGO_TARGET_I686_UNKNOWN_LINUX_GNU_LINKER="clang-21"
export CC_i686_unknown_linux_gnu="clang-21"
export AR_i686_unknown_linux_gnu="llvm-ar-21"
export CFLAGS_i686_unknown_linux_gnu="-O3 -flto"

cargo build -Z build-std=std,panic_abort --release --locked --target i686-unknown-linux-gnu
cp target/i686-unknown-linux-gnu/release/TinyWiiBackupManager .
