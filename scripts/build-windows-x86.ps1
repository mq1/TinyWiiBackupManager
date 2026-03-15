# SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
# SPDX-License-Identifier: GPL-3.0-only

$Env:RUSTC_BOOTSTRAP = "1"

$Env:CARGO_TARGET_I686_PC_WINDOWS_MSVC_RUSTFLAGS = "-C target-feature=+crt-static -L native=VC-LTL-Binary\TargetPlatform\10.0.19041.0\lib\Win32"
$Env:CC_i686_pc_windows_msvc = "clang-cl"
$Env:CFLAGS_i686_pc_windows_msvc = "/clang:-O3"

cargo build -Z build-std=std,panic_abort --release --locked --target i686-pc-windows-msvc
Copy-Item target\i686-pc-windows-msvc\release\TinyWiiBackupManager.exe .
