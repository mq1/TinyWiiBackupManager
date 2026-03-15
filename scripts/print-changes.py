#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
# SPDX-License-Identifier: GPL-3.0-only

import sys

if len(sys.argv) < 2:
    print("Usage: print-changes.py <version_name>")
    sys.exit(1)

version_name = sys.argv[1]

with open("CHANGELOG.md") as f:
    grab=False
    for line in f:
        if line.startswith(f"## [{version_name}]"): grab=True; continue
        if grab and line.startswith("## ["): break
        if grab: print(line, end="")

print(f"""<br>

<table>
    <tr>
        <td width="9999px"><strong>:arrow_down: Recommended downloads</strong></td>
    </tr>
    <tr>
        <td>
            :window: <a href="https://github.com/mq1/TinyWiiBackupManager/releases/download/{version_name}/TinyWiiBackupManager-{version_name}-windows-x86_64.zip">Windows x64 Standalone</a><br>
            :apple: <a href="https://github.com/mq1/TinyWiiBackupManager/releases/download/{version_name}/TinyWiiBackupManager-{version_name}-macos-universal.dmg">macOS Universal Binary</a><br>
            :penguin: <a href="https://github.com/mq1/TinyWiiBackupManager/releases/download/{version_name}/TinyWiiBackupManager-{version_name}-linux-x86_64.AppImage">Linux x86_64 AppImage</a>
        </td>
    </tr>
</table>""")

