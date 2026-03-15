#!/bin/bash
# SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
# SPDX-License-Identifier: GPL-3.0-only

export RUSTC_BOOTSTRAP=1

export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C linker-plugin-lto -C link-arg=-fuse-ld=lld-21 -C link-arg=--target=x86_64-unknown-linux-gnu -C target-cpu=x86-64-v2"
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER="clang-21"
export CC_x86_64_unknown_linux_gnu="clang-21"
export AR_x86_64_unknown_linux_gnu="llvm-ar-21"
export CFLAGS_x86_64_unknown_linux_gnu="-O3 -flto -march=x86-64-v2"

cargo build -Z build-std=std,panic_abort --release --locked --target x86_64-unknown-linux-gnu
cp target/x86_64-unknown-linux-gnu/release/TinyWiiBackupManager .
