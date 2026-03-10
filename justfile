# SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
# SPDX-License-Identifier: GPL-3.0-only

version := `python3 -c "import tomllib; print(tomllib.load(open('Cargo.toml','rb'))['package']['version'])"`

export CARGO_TARGET_AARCH64_APPLE_DARWIN_RUSTFLAGS := "-C link-arg=-mmacosx-version-min=11.0"
export CFLAGS_aarch64_apple_darwin := "-O3 -flto"
export CARGO_TARGET_X86_64_APPLE_DARWIN_RUSTFLAGS := "-C link-arg=-mmacosx-version-min=10.13"
export CFLAGS_x86_64_apple_darwin := "-O3 -flto"

build-macos target:
  cargo build -Z build-std=std,panic_abort --release --locked --target {{ target }}

package-macos-app target:
  install -Dm0755 "target/{{ target }}/release/TinyWiiBackupManager" TinyWiiBackupManager.app/Contents/MacOS/TinyWiiBackupManager
  install -Dm0644 package/macos/TinyWiiBackupManager.icns TinyWiiBackupManager.app/Contents/Resources/TinyWiiBackupManager.icns
  install -Dm0644 package/macos/Info.plist TinyWiiBackupManager.app/Contents/Info.plist
  /usr/libexec/PlistBuddy -c "Add :CFBundleShortVersionString {{ version }}" TinyWiiBackupManager.app/Contents/Info.plist

