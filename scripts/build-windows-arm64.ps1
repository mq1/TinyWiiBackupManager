# SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
# SPDX-License-Identifier: GPL-3.0-only

$Env:RUSTC_BOOTSTRAP = "1"

$Env:CARGO_TARGET_AARCH64_PC_WINDOWS_MSVC_RUSTFLAGS = "-C target-feature=+crt-static -C linker-plugin-lto -L native=VC-LTL-Binary\TargetPlatform\10.0.19041.0\lib\ARM64"
$Env:CARGO_TARGET_AARCH64_PC_WINDOWS_MSVC_LINKER = "lld-link"
$Env:CC_aarch64_pc_windows_msvc = "clang-cl"
$Env:AR_aarch64_pc_windows_msvc = "llvm-lib"
$Env:CFLAGS_aarch64_pc_windows_msvc = "/clang:-O3 /clang:-flto /clang:-fuse-ld=lld-link"

cargo build -Z build-std=std,panic_abort --release --locked --target aarch64-pc-windows-msvc
Copy-Item target\aarch64-pc-windows-msvc\release\TinyWiiBackupManager.exe .
