#!/bin/bash
# SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
# SPDX-License-Identifier: GPL-3.0-only

komac update mq1.TinyWiiBackupManager \
    --version $1 \
    --urls \
        https://github.com/mq1/TinyWiiBackupManager/releases/download/v$1/TinyWiiBackupManager-v$1-windows-x64.exe \
        https://github.com/mq1/TinyWiiBackupManager/releases/download/v$1/TinyWiiBackupManager-v$1-windows-x86.exe \
        https://github.com/mq1/TinyWiiBackupManager/releases/download/v$1/TinyWiiBackupManager-v$1-windows-arm64.exe \
    --submit

