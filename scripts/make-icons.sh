#!/bin/bash

# Stop the script if any command fails
set -e

# --- Universal Setup ---
# Define variables for convenience
SOURCE_ICON="./logo.png"
APP_NAME="tiny-wii-backup-manager"
ASSETS_DIR="./assets"

# Clean up previous assets and create the main directory
echo "Setting up asset directory..."
rm -rf "${ASSETS_DIR}"
mkdir -p "${ASSETS_DIR}"

# --- Linux Icon Theme Generation ---
echo "-> Generating Linux icon theme..."
for size in 32 48 64 128 256 512; do
    mkdir -p "${ASSETS_DIR}/linux/${size}x${size}"
    magick "${SOURCE_ICON}" -resize "${size}x${size}" \
        "${ASSETS_DIR}/linux/${size}x${size}/${APP_NAME}.png"
done

# --- macOS .icns File Generation ---
echo "-> Generating macOS .icns file..."
# Create a temporary directory for the icon set
ICONSET_DIR="${ASSETS_DIR}/macos/${APP_NAME}.iconset"
mkdir -p "${ICONSET_DIR}"

# Use `sips` to create all the required resolutions
sips -z 16 16     "${SOURCE_ICON}" --out "${ICONSET_DIR}/icon_16x16.png"
sips -z 32 32     "${SOURCE_ICON}" --out "${ICONSET_DIR}/icon_16x16@2x.png"
sips -z 32 32     "${SOURCE_ICON}" --out "${ICONSET_DIR}/icon_32x32.png"
sips -z 64 64     "${SOURCE_ICON}" --out "${ICONSET_DIR}/icon_32x32@2x.png"
sips -z 128 128   "${SOURCE_ICON}" --out "${ICONSET_DIR}/icon_128x128.png"
sips -z 256 256   "${SOURCE_ICON}" --out "${ICONSET_DIR}/icon_128x128@2x.png"
sips -z 256 256   "${SOURCE_ICON}" --out "${ICONSET_DIR}/icon_256x256.png"
sips -z 512 512   "${SOURCE_ICON}" --out "${ICONSET_DIR}/icon_256x256@2x.png"
sips -z 512 512   "${SOURCE_ICON}" --out "${ICONSET_DIR}/icon_512x512.png"
sips -z 1024 1024 "${SOURCE_ICON}" --out "${ICONSET_DIR}/icon_512x512@2x.png"

# Use `iconutil` to convert the icon set into a single .icns file
iconutil -c icns "${ICONSET_DIR}" -o "${ASSETS_DIR}/macos/${APP_NAME}.icns"

# Clean up the temporary icon set directory
rm -rf "${ICONSET_DIR}"

# --- Windows .ico File Generation ---
echo "-> Generating Windows .ico file..."
mkdir -p "${ASSETS_DIR}/windows"
magick "${SOURCE_ICON}" -define icon:auto-resize=256,48,32,16 \
    "${ASSETS_DIR}/windows/icon.ico"

echo "✅ All icon assets generated in ${ASSETS_DIR}"
