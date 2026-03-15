# SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
# SPDX-License-Identifier: GPL-3.0-only

$Env:RUSTC_BOOTSTRAP = "1"

$Env:CARGO_TARGET_I686_WIN7_WINDOWS_MSVC_RUSTFLAGS = "-C target-feature=+crt-static -l dylib=ole32 -L native=VC-LTL-Binary\TargetPlatform\5.1.2600.0\lib\Win32 -C link-arg=YY-Thunks-Objs\objs\x86\YY_Thunks_for_WinXP.obj -C link-arg=/SUBSYSTEM:WINDOWS,5.1"
$Env:CC_i686_win7_windows_msvc = "clang-cl"
$Env:CFLAGS_i686_win7_windows_msvc = "/clang:-O3"

cargo build -Z build-std=std,panic_abort --release --locked --target i686-win7-windows-msvc
Copy-Item target\i686-win7-windows-msvc\release\TinyWiiBackupManager.exe .
