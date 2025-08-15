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
BUNDLE_IDENTIFIER="com.github.mq1.${APP_NAME}"

DIST_DIR="./dist"
ASSETS_DIR="./assets"
APP_BUNDLE_NAME="${PRODUCT_NAME}.app"

# 1. Define targets and paths
X86_64_TARGET="x86_64-apple-darwin"
AARCH64_TARGET="aarch64-apple-darwin"
UNIVERSAL_DIR="target/universal/release"
UNIVERSAL_EXE="${UNIVERSAL_DIR}/${APP_NAME}"

# 2. Ensure toolchains are installed
echo "Ensuring required toolchains are installed..."
rustup target add ${X86_64_TARGET}
rustup target add ${AARCH64_TARGET}

# 3. Build for each architecture
echo "Building for Intel (x86_64)..."
cargo build --release --target ${X86_64_TARGET}
echo "Building for Apple Silicon (aarch64)..."
cargo build --release --target ${AARCH64_TARGET}

# 4. Combine binaries with lipo
echo "Creating Universal 2 binary with lipo..."
mkdir -p "${UNIVERSAL_DIR}"
lipo -create \
  "target/${X86_64_TARGET}/release/${APP_NAME}" \
  "target/${AARCH64_TARGET}/release/${APP_NAME}" \
  -output "${UNIVERSAL_EXE}"

# 5. Assemble the .app bundle
echo "Assembling .app bundle..."
rm -rf "${APP_BUNDLE_NAME}"
mkdir -p "${APP_BUNDLE_NAME}/Contents/MacOS"
mkdir -p "${APP_BUNDLE_NAME}/Contents/Resources"
cp "${UNIVERSAL_EXE}" "${APP_BUNDLE_NAME}/Contents/MacOS/"

echo "Copying pre-generated .icns file..."
cp "${ASSETS_DIR}/macos/${APP_NAME}.icns" "${APP_BUNDLE_NAME}/Contents/Resources/"

echo "Creating Info.plist..."
cat > "${APP_BUNDLE_NAME}/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>CFBundleName</key>
	<string>${PRODUCT_NAME}</string>
	<key>CFBundleDisplayName</key>
	<string>${PRODUCT_NAME}</string>
	<key>CFBundleIdentifier</key>
	<string>${BUNDLE_IDENTIFIER}</string>
	<key>CFBundleVersion</key>
	<string>${VERSION}</string>
	<key>CFBundleShortVersionString</key>
	<string>${VERSION}</string>
	<key>CFBundlePackageType</key>
	<string>APPL</string>
	<key>CFBundleExecutable</key>
	<string>${APP_NAME}</string>
	<key>CFBundleIconFile</key>
	<string>${APP_NAME}</string>
	<key>LSMinimumSystemVersion</key>
	<string>11.0</string>
	<key>NSHighResolutionCapable</key>
	<true/>
</dict>
</plist>
EOF

# 6. Create and finalize the DMG
echo "Creating DMG..."
FINAL_DMG_PATH="${DIST_DIR}/${SHORT_NAME}-${VERSION}-MacOS-Universal2.dmg"
mkdir -p "${DIST_DIR}"
npx --yes create-dmg "${APP_BUNDLE_NAME}" "${DIST_DIR}" --overwrite || true

echo "Renaming artifact..."
mv "${DIST_DIR}/${PRODUCT_NAME} ${VERSION}.dmg" "${FINAL_DMG_PATH}"

echo "Cleaning up intermediate .app bundle..."
rm -rf "${APP_BUNDLE_NAME}"
echo "✅ DMG created in ${DIST_DIR} directory"