# SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
# SPDX-License-Identifier: GPL-3.0-only

$ErrorActionPreference = "Stop"

param (
  [string]$Platform,
  [string]$Arch
)

$Env:RUSTC_BOOTSTRAP = "1"

switch ($Platform) {
  "windows-legacy" {
    switch ($Arch) {
      "x86" {
        $Target = "i686-win7-windows-msvc"
        $Env:CARGO_TARGET_I686_WIN7_WINDOWS_MSVC_RUSTFLAGS = "-C target-feature=+crt-static -l dylib=ole32 -L native=VC-LTL-Binary\TargetPlatform\5.1.2600.0\lib\Win32 -C link-arg=YY-Thunks-Objs\objs\x86\YY_Thunks_for_WinXP.obj -C link-arg=/SUBSYSTEM:WINDOWS,5.1"
        $Env:CC_i686_win7_windows_msvc = "clang-cl"
        $Env:CFLAGS_i686_win7_windows_msvc = "/clang:-O3"
      }
      "x86_64" {
        $Target = "x86_64-win7-windows-msvc"
        $Env:CARGO_TARGET_X86_64_WIN7_WINDOWS_MSVC_RUSTFLAGS = "-C target-feature=+crt-static -l dylib=ole32 -L native=VC-LTL-Binary\TargetPlatform\5.2.3790.0\lib\x64 -C link-arg=YY-Thunks-Objs\objs\x64\YY_Thunks_for_WinXP.obj -C link-arg=/SUBSYSTEM:WINDOWS,5.2"
        $Env:CC_x86_64_win7_windows_msvc = "clang-cl"
        $Env:CFLAGS_x86_64_win7_windows_msvc = "/clang:-O3"
      }
    }
  }
  Default {
    switch ($Arch) {
      "arm64" {
        $Target = "aarch64-pc-windows-msvc"
        $Env:CARGO_TARGET_AARCH64_PC_WINDOWS_MSVC_RUSTFLAGS = "-C target-feature=+crt-static -C linker-plugin-lto -L native=VC-LTL-Binary\TargetPlatform\10.0.19041.0\lib\ARM64"
        $Env:CARGO_TARGET_AARCH64_PC_WINDOWS_MSVC_LINKER = "lld-link"
        $Env:CC_aarch64_pc_windows_msvc = "clang-cl"
        $Env:AR_aarch64_pc_windows_msvc = "llvm-lib"
        $Env:CFLAGS_aarch64_pc_windows_msvc = "/clang:-O3 /clang:-flto /clang:-fuse-ld=lld-link"
      }
      "x86" {
        $Target = "i686-pc-windows-msvc"
        $Env:CARGO_TARGET_I686_PC_WINDOWS_MSVC_RUSTFLAGS = "-C target-feature=+crt-static -L native=VC-LTL-Binary\TargetPlatform\10.0.19041.0\lib\Win32"
        $Env:CC_i686_pc_windows_msvc = "clang-cl"
        $Env:CFLAGS_i686_pc_windows_msvc = "/clang:-O3"
      }
      "x86_64" {
        $Target = "x86_64-pc-windows-msvc"
        $Env:CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS = "-C target-feature=+crt-static -C linker-plugin-lto -L native=VC-LTL-Binary\TargetPlatform\10.0.19041.0\lib\x64"
        $Env:CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER = "lld-link"
        $Env:CC_x86_64_pc_windows_msvc = "clang-cl"
        $Env:AR_x86_64_pc_windows_msvc = "llvm-lib"
        $Env:CFLAGS_x86_64_pc_windows_msvc = "/clang:-O3 /clang:-flto /clang:-fuse-ld=lld-link"
      }
      "x86_64-v2" {
        $Target = "x86_64-pc-windows-msvc"
        $Env:CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS = "-C target-cpu=x86-64-v2 -C target-feature=+crt-static -C linker-plugin-lto -L native=VC-LTL-Binary\TargetPlatform\10.0.19041.0\lib\x64"
        $Env:CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER = "lld-link"
        $Env:CC_x86_64_pc_windows_msvc = "clang-cl"
        $Env:AR_x86_64_pc_windows_msvc = "llvm-lib"
        $Env:CFLAGS_x86_64_pc_windows_msvc = "/clang:-march=x86-64-v2 /clang:-O3 /clang:-flto /clang:-fuse-ld=lld-link"
      }
      "x86_64-v3" {
        $Target = "x86_64-pc-windows-msvc"
        $Env:CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS = "-C target-cpu=x86-64-v3 -C target-feature=+crt-static -C linker-plugin-lto -L native=VC-LTL-Binary\TargetPlatform\10.0.19041.0\lib\x64"
        $Env:CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER = "lld-link"
        $Env:CC_x86_64_pc_windows_msvc = "clang-cl"
        $Env:AR_x86_64_pc_windows_msvc = "llvm-lib"
        $Env:CFLAGS_x86_64_pc_windows_msvc = "/clang:-march=x86-64-v3 /clang:-O3 /clang:-flto /clang:-fuse-ld=lld-link"
      }
    }
  }
}

cargo build -Z build-std=std,panic_abort --release --locked --target $Target
Copy-Item "target\$Target\release\TinyWiiBackupManager.exe" .
