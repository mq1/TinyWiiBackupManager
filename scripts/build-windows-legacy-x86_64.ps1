# SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
# SPDX-License-Identifier: GPL-3.0-only

$Env:RUSTC_BOOTSTRAP = "1"

$Env:CARGO_TARGET_X86_64_WIN7_WINDOWS_MSVC_RUSTFLAGS = "-C target-feature=+crt-static -l dylib=ole32 -L native=VC-LTL-Binary\TargetPlatform\5.2.3790.0\lib\x64 -C link-arg=YY-Thunks-Objs\objs\x64\YY_Thunks_for_WinXP.obj -C link-arg=/SUBSYSTEM:WINDOWS,5.2"
$Env:CC_x86_64_win7_windows_msvc = "clang-cl"
$Env:CFLAGS_x86_64_win7_windows_msvc = "/clang:-O3"

cargo build -Z build-std=std,panic_abort --release --locked --target x86_64-win7-windows-msvc
Copy-Item target\x86_64-win7-windows-msvc\release\TinyWiiBackupManager.exe .
