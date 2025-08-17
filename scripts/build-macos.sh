#!/bin/zsh

# --- Zsh Idiomatic Options ---
setopt errexit nounset pipefail

# --- Load Zsh Utilities ---
# zargs is part of the zutil module and must be loaded before use.
autoload -U zargs

# --- Configuration ---
echo "Reading configuration from Cargo.toml..."
local APP_NAME=$(yq -r '.package.name' Cargo.toml)
local PRODUCT_NAME=$(yq -r '.package.metadata.winres.ProductName' Cargo.toml)
local SHORT_NAME=$(yq -r '.package.metadata.short_name' Cargo.toml)
local VERSION=$(yq -r '.package.version' Cargo.toml)
local DESCRIPTION=$(yq -r '.package.description' Cargo.toml)
local LEGAL_COPYRIGHT=$(yq -r '.package.metadata.winres.LegalCopyright' Cargo.toml)
local BUNDLE_IDENTIFIER="it.mq1.${PRODUCT_NAME}"

local DIST_DIR="./dist"
local ASSETS_DIR="./assets"
local APP_BUNDLE_NAME="${PRODUCT_NAME}.app"
local INFO_PLIST="${APP_BUNDLE_NAME}/Contents/Info.plist"

# 1. Define targets and paths
local X86_64_TARGET="x86_64-apple-darwin"
local AARCH64_TARGET="aarch64-apple-darwin"
local UNIVERSAL_DIR="target/universal/release"
local UNIVERSAL_EXE="${UNIVERSAL_DIR}/${APP_NAME}"

# 2. Build for each architecture
echo "Building for Intel (x86_64)..."
#cargo build --release --target ${X86_64_TARGET}
echo "Building for Apple Silicon (aarch64)..."
#cargo build --release --target ${AARCH64_TARGET}

# 3. Combine binaries with lipo
echo "Creating Universal 2 binary with lipo..."
mkdir -p "${UNIVERSAL_DIR}"
lipo -create \
    "target/${X86_64_TARGET}/release/${APP_NAME}" \
    "target/${AARCH64_TARGET}/release/${APP_NAME}" \
    -output "${UNIVERSAL_EXE}"

# 4. Assemble the .app bundle
echo "Assembling .app bundle..."
rm -rf "${APP_BUNDLE_NAME}"
mkdir -p "${APP_BUNDLE_NAME}/Contents/MacOS"
mkdir -p "${APP_BUNDLE_NAME}/Contents/Resources"
cp "${UNIVERSAL_EXE}" "${APP_BUNDLE_NAME}/Contents/MacOS/"

echo "Copying pre-generated .icns file..."
cp "${ASSETS_DIR}/macos/${APP_NAME}.icns" "${APP_BUNDLE_NAME}/Contents/Resources/"

echo "Creating Info.plist..."
# Define plist entries in an array with aligned columns for readability.
local -a plist_entries=(
    "CFBundleName"                         "string"    "${PRODUCT_NAME}"
    "CFBundleDisplayName"                  "string"    "${PRODUCT_NAME}"
    "CFBundleIdentifier"                   "string"    "${BUNDLE_IDENTIFIER}"
    "CFBundleVersion"                      "string"    "${VERSION}"
    "CFBundleShortVersionString"           "string"    "${VERSION}"
    "CFBundlePackageType"                  "string"    "APPL"
    "CFBundleExecutable"                   "string"    "${APP_NAME}"
    "CFBundleIconFile"                     "string"    "${APP_NAME}"
    "LSMinimumSystemVersion"               "string"    "11.0"
    "NSHumanReadableCopyright"             "string"    "${LEGAL_COPYRIGHT}"
    "NSPrincipalClass"                     "string"    "NSApplication"
    "NSHighResolutionCapable"              "bool"      "true"
    "NSSupportsAutomaticGraphicsSwitching" "bool"      "true"
)

# Create a new, empty plist file.
/usr/libexec/PlistBuddy -c "Clear" "${INFO_PLIST}" || true

# Define a small, local helper function to process one entry.
add_entry() {
    /usr/libexec/PlistBuddy -c "Add :$1 $2 '$3'" "${INFO_PLIST}"
}

# Use zargs to apply the function to the array.
zargs -n 3 -- ${plist_entries[@]} -- add_entry

# 5. Create and finalize the DMG
echo "Creating DMG..."
local FINAL_DMG_PATH="${DIST_DIR}/${SHORT_NAME}-${VERSION}-MacOS-Universal2.dmg"
mkdir -p "${DIST_DIR}"
npx --yes create-dmg "${APP_BUNDLE_NAME}" "${DIST_DIR}" --overwrite || true

echo "Renaming artifact..."
mv "${DIST_DIR}/${PRODUCT_NAME} ${VERSION}.dmg" "${FINAL_DMG_PATH}"

echo "Cleaning up intermediate .app bundle..."
rm -rf "${APP_BUNDLE_NAME}"
echo "✅ DMG created in ${DIST_DIR} directory"
