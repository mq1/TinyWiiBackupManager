#!/bin/bash

LATEST_VERSION=$(curl -sSL "https://github.com/mq1/TinyWiiBackupManager/releases/latest/download/version.txt")

_64bit_hash=$(curl -sSL "https://github.com/mq1/TinyWiiBackupManager/releases/download/v${LATEST_VERSION}/TinyWiiBackupManager-v${LATEST_VERSION}-windows-x86_64.zip" | sha256sum | cut -d' ' -f1)
_arm64_hash=$(curl -sSL "https://github.com/mq1/TinyWiiBackupManager/releases/download/v${LATEST_VERSION}/TinyWiiBackupManager-v${LATEST_VERSION}-windows-arm64.zip" | sha256sum | cut -d' ' -f1)
_32bit_hash=$(curl -sSL "https://github.com/mq1/TinyWiiBackupManager/releases/download/v${LATEST_VERSION}/TinyWiiBackupManager-v${LATEST_VERSION}-windows-x86.zip" | sha256sum | cut -d' ' -f1)

sed \
  -e "s/{{ LATEST_VERSION }}/${LATEST_VERSION}/g" \
  -e "s/{{ 64BIT_HASH }}/${_64bit_hash}/g" \
  -e "s/{{ ARM64_HASH }}/${_arm64_hash}/g" \
  -e "s/{{ 32BIT_HASH }}/${_32bit_hash}/g" \
  package/windows/scoop-template.json >bucket/TinyWiiBackupManager.json
