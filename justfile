# SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
# SPDX-License-Identifier: GPL-3.0-only

build-linux-x86_64:
  #!/bin/bash
  export RUSTC_BOOTSTRAP=1
  cargo zigbuild -Z build-std=std,panic_abort --release --locked --target x86_64-unknown-linux-gnu.2.17

build-linux-x86_64-v2:
  #!/bin/bash
  export RUSTC_BOOTSTRAP=1
  export RUSTFLAGS="-C target-cpu=x86-64-v2"
  export CFLAGS="-mcpu=x86_64_v2"
  cargo zigbuild -Z build-std=std,panic_abort --release --locked --target x86_64-unknown-linux-gnu.2.17

build-linux-x86_64-v3:
  #!/bin/bash
  export RUSTC_BOOTSTRAP=1
  export RUSTFLAGS="-C target-cpu=x86-64-v3"
  export CFLAGS="-mcpu=x86_64_v3"
  cargo zigbuild -Z build-std=std,panic_abort --release --locked --target x86_64-unknown-linux-gnu.2.17

build-linux-x86:
  #!/bin/bash
  export RUSTC_BOOTSTRAP=1
  cargo zigbuild -Z build-std=std,panic_abort --release --locked --target i686-unknown-linux-gnu.2.17

build-linux-arm64:
  #!/bin/bash
  export RUSTC_BOOTSTRAP=1
  cargo zigbuild -Z build-std=std,panic_abort --release --locked --target aarch64-unknown-linux-gnu.2.17

build-linux-armhf:
  #!/bin/bash
  export RUSTC_BOOTSTRAP=1
  cargo zigbuild -Z build-std=std,panic_abort --release --locked --target armv7-unknown-linux-gnueabihf.2.17

build-macos-arm64:
  #!/bin/bash
  export RUSTC_BOOTSTRAP=1
  export RUSTFLAGS="-C link-arg=-mmacosx-version-min=11.0"
  export CFLAGS="-O3 -flto"
  cargo build -Z build-std=std,panic_abort --release --locked --target aarch64-apple-darwin

build-macos-x86_64:
  #!/bin/bash
  export RUSTC_BOOTSTRAP=1
  export RUSTFLAGS="-C link-arg=-mmacosx-version-min=10.13"
  export CFLAGS="-O3 -flto"
  cargo build -Z build-std=std,panic_abort --release --locked --target x86_64-apple-darwin

package-macos-app-arm64:
  #!/bin/bash
  VERSION=$(python3 -c "import tomllib; print(tomllib.load(open('Cargo.toml','rb'))['package']['version'])")
  mkdir -p TinyWiiBackupManager.app/Contents/MacOS TinyWiiBackupManager.app/Contents/Resources
  cp "target/aarch64-apple-darwin/release/TinyWiiBackupManager" TinyWiiBackupManager.app/Contents/MacOS/TinyWiiBackupManager
  cp package/macos/TinyWiiBackupManager.icns TinyWiiBackupManager.app/Contents/Resources/TinyWiiBackupManager.icns
  cp package/macos/Info.plist TinyWiiBackupManager.app/Contents/Info.plist
  /usr/libexec/PlistBuddy -c "Add :CFBundleShortVersionString string $VERSION" TinyWiiBackupManager.app/Contents/Info.plist

package-macos-app-x86_64:
  #!/bin/bash
  VERSION=$(python3 -c "import tomllib; print(tomllib.load(open('Cargo.toml','rb'))['package']['version'])")
  mkdir -p TinyWiiBackupManager.app/Contents/MacOS TinyWiiBackupManager.app/Contents/Resources
  cp "target/x86_64-apple-darwin/release/TinyWiiBackupManager" TinyWiiBackupManager.app/Contents/MacOS/TinyWiiBackupManager
  cp package/macos/TinyWiiBackupManager.icns TinyWiiBackupManager.app/Contents/Resources/TinyWiiBackupManager.icns
  cp package/macos/Info.plist TinyWiiBackupManager.app/Contents/Info.plist
  /usr/libexec/PlistBuddy -c "Add :CFBundleShortVersionString string $VERSION" TinyWiiBackupManager.app/Contents/Info.plist

zip-macos-app version-name arch:
  #!/bin/bash
  mkdir out
  ditto -c -k \
    --sequesterRsrc \
    --keepParent \
    --zlibCompressionLevel 9 \
    TinyWiiBackupManager.app \
    "out/TinyWiiBackupManager-{{ version-name }}-macos-{{ arch }}.zip"

