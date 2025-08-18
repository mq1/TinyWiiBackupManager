#!/bin/bash

# Exit immediately if a command exits with a non-zero status.
set -e

# --- Configuration ---
echo "Reading configuration from Cargo.toml..."
APP_NAME=$(yq -r '.package.name' Cargo.toml)
PRODUCT_NAME=$(yq -r '.package.metadata.winres.ProductName' Cargo.toml)
SHORT_NAME=$(yq -r '.package.metadata.ShortName' Cargo.toml)
VERSION=$(yq -r '.package.version' Cargo.toml)
DESCRIPTION=$(yq -r '.package.description' Cargo.toml)

HOST_ARCH=$(uname -m)

DIST_DIR="./dist"
ASSETS_DIR="./assets"
DESKTOP_FILE="./${PRODUCT_NAME}.desktop" # Define the desktop file path

# 1. Prepare directories
mkdir -p "${DIST_DIR}"

# 2. Build Rust binary
echo "Building Rust binary..."
cargo build --release

# 3. Prepare input files for linuxdeploy
echo "Preparing input files (.desktop and icon)..."

# Create the .desktop file
touch "${DESKTOP_FILE}"

# Populate the .desktop file
desktop-file-edit \
    --set-name="${PRODUCT_NAME}" \
    --set-icon="${APP_NAME}" \
    --add-category=Utility \
    --set-comment="${DESCRIPTION}" \
    --set-key=Type --set-value=Application \
    --set-key=Exec --set-value="${APP_NAME}" \
    "${DESKTOP_FILE}"

# 4. Run linuxdeploy
echo "Running linuxdeploy..."
linuxdeploy-${HOST_ARCH}.AppImage \
    --appdir "${PRODUCT_NAME}.AppDir" \
    --executable "target/release/${APP_NAME}" \
    --desktop-file "${DESKTOP_FILE}" \
    --icon-file "${ASSETS_DIR}/linux/32x32/${APP_NAME}.png" \
    --icon-file "${ASSETS_DIR}/linux/48x48/${APP_NAME}.png" \
    --icon-file "${ASSETS_DIR}/linux/64x64/${APP_NAME}.png" \
    --icon-file "${ASSETS_DIR}/linux/128x128/${APP_NAME}.png" \
    --icon-file "${ASSETS_DIR}/linux/256x256/${APP_NAME}.png" \
    --icon-file "${ASSETS_DIR}/linux/512x512/${APP_NAME}.png" \
    --output appimage

# 5. Rename the final artifact
echo "Renaming artifact..."
mv "${PRODUCT_NAME}-${HOST_ARCH}.AppImage" "${DIST_DIR}/${SHORT_NAME}-${VERSION}-Linux-${HOST_ARCH}.AppImage"

# 6. Clean up the temporary desktop file
echo "Cleaning up intermediate files..."
rm "${DESKTOP_FILE}"
echo "✅ AppImage created in ${DIST_DIR} directory"

# 7. Define paths for the tar.gz archive
SOURCE_EXE="target/release/${APP_NAME}"
STAGED_EXE="./${SHORT_NAME}-${VERSION}-Linux-${HOST_ARCH}"
DEST_ARCHIVE="${DIST_DIR}/${SHORT_NAME}-${VERSION}-Linux-${HOST_ARCH}.tar.gz"

# 8. Stage, archive, and clean up the executable for the tar.gz
echo "Staging executable with final name..."
cp "${SOURCE_EXE}" "${STAGED_EXE}"
chmod +x "${STAGED_EXE}"

echo "Creating archive..."
tar -czf "${DEST_ARCHIVE}" "${STAGED_EXE}"

echo "Cleaning up staged executable..."
rm "${STAGED_EXE}"
echo "✅ .tar.gz archive created in ${DIST_DIR} directory"
