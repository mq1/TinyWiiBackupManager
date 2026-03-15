#!/bin/bash
# SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
# SPDX-License-Identifier: GPL-3.0-only

export RUSTC_BOOTSTRAP=1

export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_RUSTFLAGS="-C linker-plugin-lto -C link-arg=-fuse-ld=lld-21 -C link-arg=--target=armv7-unknown-linux-gnueabihf"
export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER="clang-21"
export CC_armv7_unknown_linux_gnueabihf="clang-21"
export AR_armv7_unknown_linux_gnueabihf="llvm-ar-21"
export CFLAGS_armv7_unknown_linux_gnueabihf="-O3 -flto"

cargo build -Z build-std=std,panic_abort --release --locked --target armv7-unknown-linux-gnueabihf
cp target/armv7-unknown-linux-gnueabihf/release/TinyWiiBackupManager .
