#!/usr/bin/env bash
set -euo pipefail

VERSION="$1"

# Setup directories
cp -r assets/TinyWiiBackupManager.app .
mkdir -p dist TinyWiiBackupManager.app/Contents/MacOS

# Create the universal binary using lipo
lipo -create \
    -output TinyWiiBackupManager.app/Contents/MacOS/TinyWiiBackupManager \
    bin-aarch64-apple-darwin/TinyWiiBackupManager bin-x86_64-apple-darwin/TinyWiiBackupManager

# Set correct binary permissions
chmod 755 TinyWiiBackupManager.app/Contents/MacOS/TinyWiiBackupManager

# Set version string
sed -i "" "s/{{ version }}/${VERSION}/" TinyWiiBackupManager.app/Contents/Info.plist
        
# Create DMG (assets from sindresorhus/create-dmg)
create-dmg \
    --volname "TinyWiiBackupManager" \
    --volicon "assets/dmg-icon.icns" \
    --background "assets/dmg-background.png" \
    --window-size 660 400 \
    --icon-size 160 \
    --icon "TinyWiiBackupManager.app/Contents/Resources/AppIcon.icns" 180 170 \
    --hide-extension "TinyWiiBackupManager/TinyWiiBackupManager.app" \
    --app-drop-link 480 170 \
    --format ULMO \
    --filesystem APFS \
    --skip-jenkins \
    "dist/TinyWiiBackupManager_${VERSION}_Universal.dmg" \
    "TinyWiiBackupManager.app"
