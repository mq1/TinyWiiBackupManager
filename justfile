# SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
# SPDX-License-Identifier: GPL-3.0-only

version := `cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version'`

build-linux-x86_64:
  #!/bin/bash
  export RUSTC_BOOTSTRAP=1
  cargo zigbuild -Z build-std=std,panic_abort --release --locked --target x86_64-unknown-linux-gnu.2.17
  cp target/x86_64-unknown-linux-gnu.2.17/release/TinyWiiBackupManager .

build-linux-x86_64-v2:
  #!/bin/bash
  export RUSTC_BOOTSTRAP=1
  export RUSTFLAGS="-C target-cpu=x86-64-v2"
  export CFLAGS="-mcpu=x86_64_v2"
  cargo zigbuild -Z build-std=std,panic_abort --release --locked --target x86_64-unknown-linux-gnu.2.17
  cp target/x86_64-unknown-linux-gnu.2.17/release/TinyWiiBackupManager .

build-linux-x86_64-v3:
  #!/bin/bash
  export RUSTC_BOOTSTRAP=1
  export RUSTFLAGS="-C target-cpu=x86-64-v3"
  export CFLAGS="-mcpu=x86_64_v3"
  cargo zigbuild -Z build-std=std,panic_abort --release --locked --target x86_64-unknown-linux-gnu.2.17
  cp target/x86_64-unknown-linux-gnu.2.17/release/TinyWiiBackupManager .

build-linux-x86:
  #!/bin/bash
  export RUSTC_BOOTSTRAP=1
  cargo zigbuild -Z build-std=std,panic_abort --release --locked --target i686-unknown-linux-gnu.2.17
  cp target/i686-unknown-linux-gnu.2.17/release/TinyWiiBackupManager .

build-linux-arm64:
  #!/bin/bash
  export RUSTC_BOOTSTRAP=1
  cargo zigbuild -Z build-std=std,panic_abort --release --locked --target aarch64-unknown-linux-gnu.2.17
  cp target/aarch64-unknown-linux-gnu.2.17/release/TinyWiiBackupManager .

build-linux-armhf:
  #!/bin/bash
  export RUSTC_BOOTSTRAP=1
  cargo zigbuild -Z build-std=std,panic_abort --release --locked --target armv7-unknown-linux-gnueabihf.2.17
  cp target/armv7-unknown-linux-gnueabihf.2.17/release/TinyWiiBackupManager .

package-linux-tarball version-name arch:
  #!/bin/bash
  mkdir -p dist
  tar \
    -I 'gzip -9' \
    --owner=0 \
    --group=0 \
    --mode=0755 \
    -cvf "dist/TinyWiiBackupManager-{{ version-name }}-linux-{{ arch }}.tar.gz" \
    TinyWiiBackupManager

package-linux-appdir:
  #!/bin/bash
  cp -r package/linux TinyWiiBackupManager.AppDir
  install -Dm0755 TinyWiiBackupManager TinyWiiBackupManager.AppDir/usr/bin/TinyWiiBackupManager

package-linux-appimage version-name arch appimagetool appimage-arch:
  #!/bin/bash
  export VERSION={{ version }}
  export ARCH={{ appimage-arch }}
  mkdir -p dist
  {{ appimagetool }} \
    -u "gh-releases-zsync|mq1|TinyWiiBackupManager|latest|*{{ arch }}.AppImage.zsync" \
    TinyWiiBackupManager.AppDir \
    dist/TinyWiiBackupManager-{{ version-name }}-linux-{{ arch }}.AppImage
  cp *.zsync dist/

build-macos-arm64:
  #!/bin/bash
  export RUSTC_BOOTSTRAP=1
  export RUSTFLAGS="-C link-arg=-mmacosx-version-min=11.0"
  export CFLAGS="-O3 -flto"
  cargo build -Z build-std=std,panic_abort --release --locked --target aarch64-apple-darwin
  cp target/aarch64-apple-darwin/release/TinyWiiBackupManager .

build-macos-x86_64:
  #!/bin/bash
  export RUSTC_BOOTSTRAP=1
  export RUSTFLAGS="-C link-arg=-mmacosx-version-min=10.13"
  export CFLAGS="-O3 -flto"
  cargo build -Z build-std=std,panic_abort --release --locked --target x86_64-apple-darwin
  cp target/x86_64-apple-darwin/release/TinyWiiBackupManager .

package-macos-app:
  #!/bin/bash
  mkdir -p TinyWiiBackupManager.app/Contents/MacOS TinyWiiBackupManager.app/Contents/Resources
  cp TinyWiiBackupManager TinyWiiBackupManager.app/Contents/MacOS/TinyWiiBackupManager
  cp package/macos/TinyWiiBackupManager.icns TinyWiiBackupManager.app/Contents/Resources/TinyWiiBackupManager.icns
  cp package/macos/Info.plist TinyWiiBackupManager.app/Contents/Info.plist
  /usr/libexec/PlistBuddy -c "Add :CFBundleShortVersionString string {{ version }}" TinyWiiBackupManager.app/Contents/Info.plist

zip-macos-app version-name arch:
  #!/bin/bash
  mkdir out
  ditto -c -k \
    --sequesterRsrc \
    --keepParent \
    --zlibCompressionLevel 9 \
    TinyWiiBackupManager.app \
    "out/TinyWiiBackupManager-{{ version-name }}-macos-{{ arch }}.zip"

