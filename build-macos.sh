#|/bin/zsh
set -e

cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin

rm -rf TinyWiiBackupManager
mkdir -p TinyWiiBackupManager/TinyWiiBackupManager.app/Contents/MacOS
lipo target/x86_64-apple-darwin/release/tiny-wii-backup-manager target/aarch64-apple-darwin/release/tiny-wii-backup-manager -create -output TinyWiiBackupManager/TinyWiiBackupManager.app/Contents/MacOS/tiny-wii-backup-manager
mkdir -p TinyWiiBackupManager/TinyWiiBackupManager.app/Contents/Resources
cp assets/TinyWiiBackupManager.icns TinyWiiBackupManager/TinyWiiBackupManager.app/Contents/Resources/
tee -a TinyWiiBackupManager/TinyWiiBackupManager.app/Contents/Info.plist << END
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple Computer//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleDevelopmentRegion</key>
  <string>English</string>
  <key>CFBundleDisplayName</key>
  <string>TinyWiiBackupManager</string>
  <key>CFBundleExecutable</key>
  <string>tiny-wii-backup-manager</string>
  <key>CFBundleIconFile</key>
  <string>TinyWiiBackupManager.icns</string>
  <key>CFBundleIdentifier</key>
  <string>eu.mq1.TinyWiiBackupManager</string>
  <key>CFBundleInfoDictionaryVersion</key>
  <string>6.0</string>
  <key>CFBundleName</key>
  <string>TinyWiiBackupManager</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleShortVersionString</key>
  <string>$1</string>
  <key>CFBundleVersion</key>
  <string>20231026.095229</string>
  <key>CSResourcesFileMapped</key>
  <true/>
  <key>LSApplicationCategoryType</key>
  <string>public.app-category.games</string>
  <key>LSRequiresCarbon</key>
  <true/>
  <key>NSHighResolutionCapable</key>
  <true/>
  <key>NSHumanReadableCopyright</key>
  <string>Copyright (c) 2023 Manuel Quarneti</string>
</dict>
</plist>
END
ln -sf /Applications TinyWiiBackupManager/Applications

rm -f "TinyWiiBackupManager-$1-MacOS-Universal2.dmg"
hdiutil create "TinyWiiBackupManager-$1-MacOS-Universal2.dmg" -volname TinyWiiBackupManager -fs HFS+ -srcfolder TinyWiiBackupManager -format UDZO
rm -rf TinyWiiBackupManager
