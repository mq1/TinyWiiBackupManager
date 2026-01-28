#!/bin/bash

komac update mq1.TinyWiiBackupManager \
  --version "${1}" \
  --urls \
  "https://github.com/mq1/TinyWiiBackupManager/releases/download/v${1}/TinyWiiBackupManager-v${1}-windows-x86_64-setup.exe" \
  "https://github.com/mq1/TinyWiiBackupManager/releases/download/v${1}/TinyWiiBackupManager-v${1}-windows-arm64-setup.exe" \
  "https://github.com/mq1/TinyWiiBackupManager/releases/download/v${1}/TinyWiiBackupManager-v${1}-windows-x86-setup.exe" \
  --submit
