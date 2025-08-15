#!/bin/bash

# Exit immediately if a command exits with a non-zero status.
set -e

# --- Configuration ---
echo "Reading configuration from Cargo.toml..."
APP_NAME=$(yq -r '.package.name' Cargo.toml)
PRODUCT_NAME=$(yq -r '.package.metadata.winres.ProductName' Cargo.toml)
SHORT_NAME=$(yq -r '.package.metadata.short_name' Cargo.toml)
VERSION=$(yq -r '.package.version' Cargo.toml)
DESCRIPTION=$(yq -r '.package.description' Cargo.toml)

HOST_ARCH=$(uname -m)

DIST_DIR="./dist"
ASSETS_DIR="./assets"
INPUT_DIR="./tmp-linux-bundle-assets"

# 1. Prepare directories
mkdir -p "${DIST_DIR}"
rm -rf "${INPUT_DIR}"
mkdir -p "${INPUT_DIR}"

# 2. Download linuxdeploy and its AppImage plugin
echo "Downloading linuxdeploy tools..."
wget -c "https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-${HOST_ARCH}.AppImage"
wget -c "https://github.com/linuxdeploy/linuxdeploy-plugin-appimage/releases/download/continuous/linuxdeploy-plugin-appimage-${HOST_ARCH}.AppImage"
chmod +x linuxdeploy*.AppImage

# 3. Build Rust binary
echo "Building Rust binary..."
cargo build --release

# 4. Prepare input files for linuxdeploy
echo "Preparing input files (.desktop and icon)..."
# The .desktop file must be named after the binary
cat > "${INPUT_DIR}/${APP_NAME}.desktop" <<EOF
[Desktop Entry]
Name=${PRODUCT_NAME}
Exec=${APP_NAME}
Icon=${APP_NAME}
Type=Application
Categories=Utility;
Comment=${DESCRIPTION}
EOF
# The icon file must be named after the binary
cp "${ASSETS_DIR}/linux/icons/hicolor/256x256/apps/${APP_NAME}.png" "${INPUT_DIR}/${APP_NAME}.png"

# 5. Run linuxdeploy
echo "Running linuxdeploy..."
./linuxdeploy-${HOST_ARCH}.AppImage \
    --appdir "${PRODUCT_NAME}.AppDir" \
    --executable "target/release/${APP_NAME}" \
    --desktop-file "${INPUT_DIR}/${APP_NAME}.desktop" \
    --icon-file "${INPUT_DIR}/${APP_NAME}.png" \
    --output appimage

# 6. Rename the final artifact
echo "Renaming artifact..."
# linuxdeploy creates a file like "AppName-arch.AppImage"
mv "${PRODUCT_NAME}"-"*.AppImage" "${DIST_DIR}/${SHORT_NAME}-${VERSION}-Linux-${HOST_ARCH}.AppImage"

# 7. Clean up intermediate files
echo "Cleaning up intermediate files..."
rm -rf "${INPUT_DIR}"
rm linuxdeploy*.AppImage
echo "✅ AppImage created in ${DIST_DIR} directory"

# 1. Define paths
SOURCE_EXE="target/release/${APP_NAME}"
STAGED_EXE="./${PRODUCT_NAME}"
DEST_ARCHIVE="${DIST_DIR}/${SHORT_NAME}-${VERSION}-Linux-${HOST_ARCH}.tar.gz"

# 2. Stage, archive, and clean up
echo "Staging executable with final name..."
cp "${SOURCE_EXE}" "${STAGED_EXE}"
chmod +x "${STAGED_EXE}"

echo "Creating archive..."
tar -czf "${DEST_ARCHIVE}" "${STAGED_EXE}"

echo "Cleaning up staged executable..."
rm "${STAGED_EXE}"
echo "✅ .tar.gz archive created in ${DIST_DIR} directory"