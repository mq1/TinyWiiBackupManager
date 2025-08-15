# Stop the script if any command fails
$ErrorActionPreference = "Stop"

# --- Configuration ---
Write-Host "Reading configuration from Cargo.toml..."
$APP_NAME = (yq -r '.package.name' Cargo.toml)
$FANCY_APP_NAME = "TinyWiiBackupManager"
$VERSION = (yq -r '.package.version' Cargo.toml)

$HOST_ARCH = $env:PROCESSOR_ARCHITECTURE

$PREFIX = "TWBM"
$DIST_DIR = ".\dist"
$ASSETS_DIR = ".\assets"

# --- Build .zip Archive ---
Write-Host ""
Write-Host "--- Building Windows package for $($APP_NAME) v$($VERSION) (arch: $($HOST_ARCH)) ---"

# 1. Prepare directories
New-Item -ItemType Directory -Force -Path $DIST_DIR

# 2. Build Rust binary (will embed icon via build.rs)
Write-Host "Building Rust binary..."
cargo build --release

# 3. Define paths
$SOURCE_EXE = ".\target\release\$($APP_NAME).exe"
$STAGED_EXE = ".\$($FANCY_APP_NAME).exe"
$DEST_ZIP = "$($DIST_DIR)\$($PREFIX)-$($VERSION)-Windows-$($HOST_ARCH).zip"

# 4. Stage, archive, and clean up
Write-Host "Staging executable with final name..."
Copy-Item -Path $SOURCE_EXE -Destination $STAGED_EXE

Write-Host "Zipping the executable..."
Compress-Archive -Path $STAGED_EXE -DestinationPath $DEST_ZIP -Force

Write-Host "Cleaning up staged executable..."
Remove-Item $STAGED_EXE
Write-Host "✅ Windows package created in $($DIST_DIR) directory"