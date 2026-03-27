#!/bin/bash
# SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
# SPDX-License-Identifier: GPL-3.0-only

set -euo pipefail

ARCH=$1

export RUSTC_BOOTSTRAP=1

case $ARCH in
arm64)
  export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C linker-plugin-lto -C link-arg=-fuse-ld=lld-21 -C link-arg=--target=aarch64-unknown-linux-gnu"
  export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER="clang-21"
  export CC_aarch64_unknown_linux_gnu="clang-21"
  export AR_aarch64_unknown_linux_gnu="llvm-ar-21"
  export CFLAGS_aarch64_unknown_linux_gnu="-O3 -flto"
  TARGET="aarch64-unknown-linux-gnu"
  ;;
armhf)
  export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_RUSTFLAGS="-C linker-plugin-lto -C link-arg=-fuse-ld=lld-21 -C link-arg=--target=armv7-unknown-linux-gnueabihf"
  export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER="clang-21"
  export CC_armv7_unknown_linux_gnueabihf="clang-21"
  export AR_armv7_unknown_linux_gnueabihf="llvm-ar-21"
  export CFLAGS_unknown_linux_gnueabihf="-O3 -flto"
  export PKG_CONFIG="arm-linux-gnueabihf-pkg-config"
  export PKG_CONFIG_PATH="/usr/lib/arm-linux-gnueabihf/pkgconfig"
  TARGET="armv7-unknown-linux-gnueabihf"
  ;;
x86)
  export CARGO_TARGET_I686_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C linker-plugin-lto -C link-arg=-fuse-ld=lld-21 -C link-arg=--target=i686-unknown-linux-gnu"
  export CARGO_TARGET_I686_UNKNOWN_LINUX_GNU_LINKER="clang-21"
  export CC_i686_unknown_linux_gnu="clang-21"
  export AR_i686_unknown_linux_gnu="llvm-ar-21"
  export CFLAGS_i686_unknown_linux_gnu="-O3 -flto"
  export PKG_CONFIG="i686-linux-gnu-pkg-config"
  export PKG_CONFIG_PATH="/usr/lib/i386-linux-gnu/pkgconfig"
  TARGET="i686-unknown-linux-gnu"
  ;;
x86_64)
  export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C linker-plugin-lto -C link-arg=-fuse-ld=lld-21 -C link-arg=--target=x86_64-unknown-linux-gnu"
  export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER="clang-21"
  export CC_x86_64_unknown_linux_gnu="clang-21"
  export AR_x86_64_unknown_linux_gnu="llvm-ar-21"
  export CFLAGS_x86_64_unknown_linux_gnu="-O3 -flto"
  TARGET="x86_64-unknown-linux-gnu"
  ;;
x86_64-v2)
  export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C linker-plugin-lto -C link-arg=-fuse-ld=lld-21 -C link-arg=--target=x86_64-unknown-linux-gnu -C target-cpu=x86-64-v2"
  export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER="clang-21"
  export CC_x86_64_unknown_linux_gnu="clang-21"
  export AR_x86_64_unknown_linux_gnu="llvm-ar-21"
  export CFLAGS_x86_64_unknown_linux_gnu="-O3 -flto -march=x86-64-v2"
  TARGET="x86_64-unknown-linux-gnu"
  ;;
x86_64-v3)
  export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C linker-plugin-lto -C link-arg=-fuse-ld=lld-21 -C link-arg=--target=x86_64-unknown-linux-gnu -C target-cpu=x86-64-v3"
  export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER="clang-21"
  export CC_x86_64_unknown_linux_gnu="clang-21"
  export AR_x86_64_unknown_linux_gnu="llvm-ar-21"
  export CFLAGS_x86_64_unknown_linux_gnu="-O3 -flto -march=x86-64-v3"
  TARGET="x86_64-unknown-linux-gnu"
  ;;
esac

cargo build -Z build-std=std,panic_abort --release --locked --target $TARGET --no-default-features
cp target/$TARGET/release/TinyWiiBackupManager .
