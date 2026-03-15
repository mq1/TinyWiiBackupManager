#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
# SPDX-License-Identifier: GPL-3.0-only

import sys
import urllib.request
import json

if len(sys.argv) < 2:
    print("Usage: print-scoop-manifest.py <version-name>")
    sys.exit(1)

version_name = sys.argv[1]

manifest = {
    "$schema": "https://raw.githubusercontent.com/ScoopInstaller/Scoop/master/schema.json",
    "version": version_name.removeprefix("v"),
    "description": "A tiny game backup and homebrew app manager for the Wii",
    "homepage": "https://github.com/mq1/TinyWiiBackupManager",
    "license": "GPL-3.0-only",
    "shortcuts": [["TinyWiiBackupManager.exe", "TinyWiiBackupManager"]],
    "architecture": {
        "64bit": { "url": f"https://github.com/mq1/TinyWiiBackupManager/releases/download/{version_name}/TinyWiiBackupManager-{version_name}-windows-x86_64.zip" },
        "arm64": { "url": f"https://github.com/mq1/TinyWiiBackupManager/releases/download/{version_name}/TinyWiiBackupManager-{version_name}-windows-arm64.zip" },
        "32bit": { "url": f"https://github.com/mq1/TinyWiiBackupManager/releases/download/{version_name}/TinyWiiBackupManager-{version_name}-windows-x86.zip" }
    }
}

with urllib.request.urlopen(f"https://api.github.com/repos/mq1/TinyWiiBackupManager/releases/tags/{version_name}") as response:
    data = json.load(response)

for asset in data["assets"]:
    if asset["name"] == f"TinyWiiBackupManager-{version_name}-windows-x86_64.zip":
        manifest["architecture"]["64bit"]["hash"] = asset["digest"].removeprefix("sha256:")
    elif asset["name"] == f"TinyWiiBackupManager-{version_name}-windows-arm64.zip":
        manifest["architecture"]["arm64"]["hash"] = asset["digest"].removeprefix("sha256:")
    elif asset["name"] == f"TinyWiiBackupManager-{version_name}-windows-x86.zip":
        manifest["architecture"]["32bit"]["hash"] = asset["digest"].removeprefix("sha256:")

print(json.dumps(manifest, indent=2))
