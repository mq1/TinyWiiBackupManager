# SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
# SPDX-License-Identifier: GPL-3.0-only

version := `python3 -c "import tomllib; print(tomllib.load(open('Cargo.toml','rb'))['package']['version'])"`

export RUSTC_BOOTSTRAP := "1"
export CARGO_TARGET_AARCH64_APPLE_DARWIN_RUSTFLAGS := "-C link-arg=-mmacosx-version-min=11.0"
export CFLAGS_aarch64_apple_darwin := "-O3 -flto"
export CARGO_TARGET_X86_64_APPLE_DARWIN_RUSTFLAGS := "-C link-arg=-mmacosx-version-min=10.13"
export CFLAGS_x86_64_apple_darwin := "-O3 -flto"

build-macos target:
  cargo build -Z build-std=std,panic_abort --release --locked --target {{ target }}

package-macos-app target:
  mkdir -p TinyWiiBackupManager.app/Contents/MacOS TinyWiiBackupManager.app/Contents/Resources
  cp "target/{{ target }}/release/TinyWiiBackupManager" TinyWiiBackupManager.app/Contents/MacOS/TinyWiiBackupManager
  cp package/macos/TinyWiiBackupManager.icns TinyWiiBackupManager.app/Contents/Resources/TinyWiiBackupManager.icns
  cp package/macos/Info.plist TinyWiiBackupManager.app/Contents/Info.plist
  /usr/libexec/PlistBuddy -c "Add :CFBundleShortVersionString string {{ version }}" TinyWiiBackupManager.app/Contents/Info.plist

zip-macos-app version-name dist-name:
  mkdir out
  ditto -c -k --sequesterRsrc --keepParent --zlibCompressionLevel 9 TinyWiiBackupManager.app "out/TinyWiiBackupManager-{{ version-name }}-{{ dist-name }}.zip"

