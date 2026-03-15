# SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
# SPDX-License-Identifier: GPL-3.0-only

$ErrorActionPreference = "Stop"

param (
    [string]$VersionName,
    [string]$Platform,
    [string]$Arch
)

New-Item -Path "dist" -ItemType Directory -Force
7z a -tzip -mx=9 "dist\TinyWiiBackupManager-$VersionName-$Platform-$Arch.zip" TinyWiiBackupManager.exe
