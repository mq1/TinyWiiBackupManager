#!/bin/bash

echo "Cleaning up build artifacts and generated assets..."

# Remove cargo build artifacts
cargo clean

# Remove top-level directories
rm -rf "./dist"
rm -rf "./assets"

# Remove intermediate files that might be left over if a build fails
rm -f "TinyWiiBackupManager.exe"
rm -f "TinyWiiBackupManager"
rm -f "tiny-wii-backup-manager-*.AppImage"
rm -rf "*.app"
rm -rf "*.AppDir"
rm -f "*.dmg"

echo "✅ Cleanup complete."