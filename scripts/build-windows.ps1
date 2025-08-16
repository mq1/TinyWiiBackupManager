# Stop the script if any command fails
$ErrorActionPreference = "Stop"

# --- Configuration ---
$cargoTomlContent = Get-Content -Path Cargo.toml -Raw
$cargoConfig = $cargoTomlContent | ConvertFrom-Toml
$APP_NAME = $cargoConfig.package.name
$PRODUCT_NAME = $cargoConfig.package.metadata.winres.ProductName
$SHORT_NAME = $cargoConfig.package.metadata.short_name
$VERSION = $cargoConfig.package.version

$HOST_ARCH = $env:PROCESSOR_ARCHITECTURE

$DIST_DIR = ".\dist"

# 1. Prepare directories
New-Item -ItemType Directory -Force -Path $DIST_DIR

# 2. Build Rust binary (will embed icon via build.rs)
Write-Host "Building Rust binary..."
cargo build --release

# 3. Define paths
$SOURCE_EXE = ".\target\release\$($APP_NAME).exe"
$STAGED_EXE = ".\$($PRODUCT_NAME).exe"
$DEST_ZIP = "$($DIST_DIR)\$($SHORT_NAME)-$($VERSION)-Windows-$($HOST_ARCH).zip"

# 4. Stage, archive, and clean up
Write-Host "Staging executable with final name..."
Copy-Item -Path $SOURCE_EXE -Destination $STAGED_EXE

Write-Host "Zipping the executable..."
Compress-Archive -Path $STAGED_EXE -DestinationPath $DEST_ZIP -Force

Write-Host "Cleaning up staged executable..."
Remove-Item $STAGED_EXE
Write-Host "✅ Windows package created in $($DIST_DIR) directory"