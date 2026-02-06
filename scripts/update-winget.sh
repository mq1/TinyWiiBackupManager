#!/bin/bash

komac update mq1.TinyWiiBackupManager \
  --version "$1" \
  --urls \
  "https://github.com/mq1/TinyWiiBackupManager/releases/download/v$1/TinyWiiBackupManager-v$1-windows-x86_64.zip" \
  "https://github.com/mq1/TinyWiiBackupManager/releases/download/v$1/TinyWiiBackupManager-v$1-windows-arm64.zip" \
  "https://github.com/mq1/TinyWiiBackupManager/releases/download/v$1/TinyWiiBackupManager-v$1-windows-x86.zip" \
  --submit
