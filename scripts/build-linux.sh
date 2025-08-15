#!/bin/bash

# Exit immediately if a command exits with a non-zero status.
set -e

# --- Configuration ---
echo "Reading configuration from Cargo.toml..."
APP_NAME=$(yq -r '.package.name' Cargo.toml)
FANCY_APP_NAME="TinyWiiBackupManager"
VERSION=$(yq -r '.package.version' Cargo.toml)
DESCRIPTION=$(yq -r '.package.description' Cargo.toml)

HOST_ARCH=$(uname -m)

PREFIX="TWBM"
DIST_DIR="./dist"
ASSETS_DIR="./assets"
APPDIR_NAME="${APP_NAME}.AppDir"

# Download appimagetool
rm -f "appimagetool-$HOST_ARCH.AppImage"
wget "https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-$HOST_ARCH.AppImage"
chmod +x "appimagetool-$HOST_ARCH.AppImage"

# 1. Clean up and prepare directories
rm -rf "${APPDIR_NAME}"
mkdir -p "${DIST_DIR}"

# 2. Build Rust binary
echo "Building Rust binary..."
cargo build --release

# 3. Assemble AppDir
echo "Assembling AppDir..."
mkdir -p "${APPDIR_NAME}/usr/bin"
cp "target/release/${APP_NAME}" "${APPDIR_NAME}/usr/bin/"

echo "Copying pre-generated icons..."
mkdir -p "${APPDIR_NAME}/usr/share"
cp -r "${ASSETS_DIR}/linux/icons" "${APPDIR_NAME}/usr/share/"

echo "Creating desktop file..."
mkdir -p "${APPDIR_NAME}/usr/share/applications"
cat > "${APPDIR_NAME}/usr/share/applications/${APP_NAME}.desktop" <<EOF
[Desktop Entry]
Name=${FANCY_APP_NAME}
Exec=${APP_NAME}
Icon=${APP_NAME}
Type=Application
Categories=Utility;Game;
Comment=${DESCRIPTION}
EOF

echo "Setting the AppImage file icon..."
cp "${ASSETS_DIR}/linux/icons/hicolor/256x256/apps/${APP_NAME}.png" "${APPDIR_NAME}/.DirIcon"

# 4. Run appimagetool and place artifact in dist
echo "Running appimagetool..."
./appimagetool-$HOST_ARCH.AppImage --comp gzip "${APPDIR_NAME}"
mv "${APP_NAME}-${HOST_ARCH}.AppImage" "${DIST_DIR}/${PREFIX}-${VERSION}-Linux-${HOST_ARCH}.AppImage"
rm -rf "${APPDIR_NAME}"
echo "✅ AppImage created in ${DIST_DIR} directory"

# --- Build .tar.gz Archive ---
echo ""
echo "--- Building .tar.gz archive for ${APP_NAME} v${VERSION} (arch: ${HOST_ARCH}) ---"

# 1. Define paths
SOURCE_EXE="target/release/${APP_NAME}"
STAGED_EXE="./${FANCY_APP_NAME}"
DEST_ARCHIVE="${DIST_DIR}/${PREFIX}-${VERSION}-Linux-${HOST_ARCH}.tar.gz"

# 2. Stage, archive, and clean up
echo "Staging executable with final name..."
cp "${SOURCE_EXE}" "${STAGED_EXE}"
chmod +x "${STAGED_EXE}"

echo "Creating archive..."
tar -czf "${DEST_ARCHIVE}" "${STAGED_EXE}"

echo "Cleaning up staged executable..."
rm "${STAGED_EXE}"
echo "✅ .tar.gz archive created in ${DIST_DIR} directory"