#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
# SPDX-License-Identifier: GPL-3.0-only

import sys
import subprocess

if len(sys.argv) < 2:
    print("Usage: update-winget.py <version-name>")
    sys.exit(1)

version_name = sys.argv[1]

subprocess.check_output([
    "komac",
    "update",
    "mq1.TinyWiiBackupManager",
    "--version",
    version_name.removeprefix("v"),
    "--urls",
    f"https://github.com/mq1/TinyWiiBackupManager/releases/download/{version_name}/TinyWiiBackupManager-{version_name}-windows-x86_64.zip",
    f"https://github.com/mq1/TinyWiiBackupManager/releases/download/{version_name}/TinyWiiBackupManager-{version_name}-windows-arm64.zip",
    f"https://github.com/mq1/TinyWiiBackupManager/releases/download/{version_name}/TinyWiiBackupManager-{version_name}-windows-x86.zip",
    "--submit"
])
