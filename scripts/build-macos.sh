#!/bin/bash

# Exit immediately if a command exits with a non-zero status.
set -e

# --- Configuration ---
echo "Reading configuration from Cargo.toml..."
APP_NAME=$(yq -r '.package.name' Cargo.toml)
PRODUCT_NAME=$(yq -r '.package.metadata.winres.ProductName' Cargo.toml)
SHORT_NAME=$(yq -r '.package.metadata.ShortName' Cargo.toml)
BUNDLE_IDENTIFIER=$(yq -r '.package.metadata.BundleIdentifier' Cargo.toml)
VERSION=$(yq -r '.package.version' Cargo.toml)
LEGAL_COPYRIGHT=$(yq -r '.package.metadata.winres.LegalCopyright' Cargo.toml)

DIST_DIR="./dist"
ASSETS_DIR="./assets"
APP_BUNDLE_NAME="${PRODUCT_NAME}.app"
INFO_PLIST="${APP_BUNDLE_NAME}/Contents/Info.plist"

# Define targets and paths
X86_64_TARGET="x86_64-apple-darwin"
AARCH64_TARGET="aarch64-apple-darwin"
UNIVERSAL_DIR="target/universal/release"
UNIVERSAL_EXE="${UNIVERSAL_DIR}/${APP_NAME}"

# Build for each architecture
echo "Building for Intel (x86_64)..."
cargo build --release --target ${X86_64_TARGET}
echo "Building for Apple Silicon (aarch64)..."
cargo build --release --target ${AARCH64_TARGET}

# Combine binaries with lipo
echo "Creating Universal 2 binary with lipo..."
mkdir -p "${UNIVERSAL_DIR}"
lipo -create \
    "target/${X86_64_TARGET}/release/${APP_NAME}" \
    "target/${AARCH64_TARGET}/release/${APP_NAME}" \
    -output "${UNIVERSAL_EXE}"

# Assemble the .app bundle
echo "Assembling .app bundle..."
rm -rf "${APP_BUNDLE_NAME}"
mkdir -p "${APP_BUNDLE_NAME}/Contents/MacOS"
mkdir -p "${APP_BUNDLE_NAME}/Contents/Resources"
cp "${UNIVERSAL_EXE}" "${APP_BUNDLE_NAME}/Contents/MacOS/"
cp "${ASSETS_DIR}/macos/${APP_NAME}.icns" "${APP_BUNDLE_NAME}/Contents/Resources/"

# Create Info.plist
echo "Creating Info.plist..."
/usr/libexec/PlistBuddy -c "Clear" "${INFO_PLIST}" || true

# Use a unique delimiter like ' • ', ensuring it doesn't appear in values.
# Add plist entries
entries=(
    "CFBundleName • string • ${PRODUCT_NAME}"
    "CFBundleDisplayName • string • ${PRODUCT_NAME}"
    "CFBundleIdentifier • string • ${BUNDLE_IDENTIFIER}"
    "CFBundleVersion • string • ${VERSION}"
    "CFBundleShortVersionString • string • ${VERSION}"
    "CFBundlePackageType • string • APPL"
    "CFBundleExecutable • string • ${APP_NAME}"
    "CFBundleIconFile • string • ${APP_NAME}"
    "LSMinimumSystemVersion • string • 11.0"
    "NSHumanReadableCopyright • string • ${LEGAL_COPYRIGHT}"
    "NSPrincipalClass • string • NSApplication"
    "NSHighResolutionCapable • bool • true"
    "NSSupportsAutomaticGraphicsSwitching • bool • true"
)

for entry in "${entries[@]}"; do
    # Change IFS to the unique delimiter
    IFS=' • ' read -r key type value <<< "$entry"
    /usr/libexec/PlistBuddy -c "Add :$key $type '$value'" "${INFO_PLIST}"
done

# Create and finalize the DMG
echo "Creating DMG..."
FINAL_DMG_PATH="${DIST_DIR}/${SHORT_NAME}-${VERSION}-MacOS-Universal2.dmg"
mkdir -p "${DIST_DIR}"
npx --yes create-dmg "${APP_BUNDLE_NAME}" "${DIST_DIR}" --overwrite || true
mv "${DIST_DIR}/${PRODUCT_NAME} ${VERSION}.dmg" "${FINAL_DMG_PATH}"

# Cleanup
rm -rf "${APP_BUNDLE_NAME}"
echo "✅ DMG created in ${DIST_DIR} directory"
