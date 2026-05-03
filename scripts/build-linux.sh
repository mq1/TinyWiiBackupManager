#!/bin/bash
# SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
# SPDX-License-Identifier: GPL-3.0-only

set -euxo pipefail

# Install LLVM
wget -O /etc/apt/trusted.gpg.d/apt.llvm.org.asc https://apt.llvm.org/llvm-snapshot.gpg.key
echo "deb http://apt.llvm.org/bullseye/ llvm-toolchain-bullseye-22 main" > /etc/apt/sources.list.d/llvm.list
apt-get update
apt-get install -y clang-22 lld-22

# Add rust-src
rustup component add rust-src

# Set up environment variables
export RUSTC_BOOTSTRAP=1
export CC=clang-22
export AR=llvm-ar-22
export CFLAGS="-O3 -flto"
export RUSTFLAGS="-C linker-plugin-lto -C link-arg=-fuse-ld=lld-22"
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=clang-22
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=clang-22
export CARGO_TARGET_I686_UNKNOWN_LINUX_GNU_LINKER=clang-22

cargo build -Z build-std=std,panic_abort --release --target $1
