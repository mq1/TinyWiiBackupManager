# SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
# SPDX-License-Identifier: GPL-3.0-only

param(
  [string]$Platform,
  [string]$Arch
)

$ErrorActionPreference = "Stop"

$Env:RUSTC_BOOTSTRAP = "1"

switch ($Platform) {
  "windows-legacy" {
    switch ($Arch) {
      "x86" {
        $Target = "i686-win7-windows-msvc"
      }
      "x86_64" {
        $Target = "x86_64-win7-windows-msvc"
      }
    }
  }
  "windows" {
    switch ($Arch) {
      "arm64" {
        $Target = "aarch64-pc-windows-msvc"
      }
      "x86" {
        $Target = "i686-pc-windows-msvc"
      }
      "x86_64" {
        $Target = "x86_64-pc-windows-msvc"
      }
      "x86_64-v2" {
        $Target = "x86_64-pc-windows-msvc"
        $Env:RUSTFLAGS = "-C target-cpu=x86-64-v2"
        $Env:CFLAGS = "/clang:-march=x86-64-v2"
      }
      "x86_64-v3" {
        $Target = "x86_64-pc-windows-msvc"
        $Env:RUSTFLAGS = "-C target-cpu=x86-64-v3"
        $Env:CFLAGS = "/clang:-march=x86-64-v3"
      }
    }
  }
}

cargo build -Z build-std=std,panic_abort --release --locked --target $Target
Copy-Item "target\$Target\release\TinyWiiBackupManager.exe" .
