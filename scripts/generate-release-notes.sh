#!/bin/bash

if ! grep -Fq "## [$1]" CHANGELOG.md; then
  exit 1
fi

awk -v t="$1" '/^## \[/ && f {exit} $0 ~ "^## \\[" t {f=1; next} f' CHANGELOG.md

cat <<EOF
<br> 

<table>
  <tr>
    <td width="9999px"><strong>:arrow_down: Recommended downloads</strong></td>
  </tr>
  <tr>
    <td>
    :window: <a href="https://github.com/mq1/TinyWiiBackupManager/releases/download/$1/TinyWiiBackupManager-$1-windows-x86_64.zip">Windows x64 Standalone</a>
    <br>
    :apple: <a href="https://github.com/mq1/TinyWiiBackupManager/releases/download/$1/TinyWiiBackupManager-$1-macos-universal.dmg">macOS Universal Binary</a>
    <br>
    :penguin: <a href="https://github.com/mq1/TinyWiiBackupManager/releases/download/$1/TinyWiiBackupManager-$1-linux-x86_64.AppImage">Linux x86_64 AppImage</a>
    </td>
  </tr>
</table>
EOF
