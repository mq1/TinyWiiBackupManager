# SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
# SPDX-License-Identifier: GPL-3.0-only

version := `cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version'`


build-linux-x86_64:
  #!/bin/bash
  set -euo pipefail
  export RUSTC_BOOTSTRAP=1
  cargo zigbuild -Z build-std=std,panic_abort --release --locked --target x86_64-unknown-linux-gnu.2.17
  cp target/x86_64-unknown-linux-gnu/release/TinyWiiBackupManager .

build-linux-x86_64-v2:
  #!/bin/bash
  set -euo pipefail
  export RUSTC_BOOTSTRAP=1
  export RUSTFLAGS="-C target-cpu=x86-64-v2"
  export CFLAGS="-mcpu=x86_64_v2"
  cargo zigbuild -Z build-std=std,panic_abort --release --locked --target x86_64-unknown-linux-gnu.2.17
  cp target/x86_64-unknown-linux-gnu/release/TinyWiiBackupManager .

build-linux-x86_64-v3:
  #!/bin/bash
  set -euo pipefail
  export RUSTC_BOOTSTRAP=1
  export RUSTFLAGS="-C target-cpu=x86-64-v3"
  export CFLAGS="-mcpu=x86_64_v3"
  cargo zigbuild -Z build-std=std,panic_abort --release --locked --target x86_64-unknown-linux-gnu.2.17
  cp target/x86_64-unknown-linux-gnu/release/TinyWiiBackupManager .

build-linux-x86:
  #!/bin/bash
  set -euo pipefail
  export RUSTC_BOOTSTRAP=1
  cargo zigbuild -Z build-std=std,panic_abort --release --locked --target i686-unknown-linux-gnu.2.17
  cp target/i686-unknown-linux-gnu/release/TinyWiiBackupManager .

build-linux-arm64:
  #!/bin/bash
  set -euo pipefail
  export RUSTC_BOOTSTRAP=1
  cargo zigbuild -Z build-std=std,panic_abort --release --locked --target aarch64-unknown-linux-gnu.2.17
  cp target/aarch64-unknown-linux-gnu/release/TinyWiiBackupManager .

build-linux-armhf:
  #!/bin/bash
  set -euo pipefail
  export RUSTC_BOOTSTRAP=1
  cargo zigbuild -Z build-std=std,panic_abort --release --locked --target armv7-unknown-linux-gnueabihf.2.17
  cp target/armv7-unknown-linux-gnueabihf/release/TinyWiiBackupManager .

package-linux-tarball version-name arch:
  #!/bin/bash
  set -euo pipefail
  mkdir -p dist
  tar \
    -I 'gzip -9' \
    --owner=0 \
    --group=0 \
    --mode=0755 \
    -cvf "dist/TinyWiiBackupManager-{{ version-name }}-linux-{{ arch }}.tar.gz" \
    TinyWiiBackupManager

package-linux-appimage version-name arch appimagetool appimage-arch:
  #!/bin/bash
  set -euo pipefail
  export VERSION={{ version }}
  export ARCH={{ appimage-arch }}
  cp -r package/linux TinyWiiBackupManager.AppDir
  install -Dm0755 TinyWiiBackupManager TinyWiiBackupManager.AppDir/usr/bin/TinyWiiBackupManager
  mkdir -p dist
  {{ appimagetool }} \
    -u "gh-releases-zsync|mq1|TinyWiiBackupManager|latest|*{{ arch }}.AppImage.zsync" \
    TinyWiiBackupManager.AppDir \
    dist/TinyWiiBackupManager-{{ version-name }}-linux-{{ arch }}.AppImage
  cp *.zsync dist/


build-windows-x86_64:
  #!pwsh
  $ErrorActionPreference = "Stop"
  $Env:RUSTC_BOOTSTRAP = "1"
  $Env:RUSTFLAGS = "-C target-feature=+crt-static -C linker-plugin-lto"
  $Env:CFLAGS = "/clang:-O3 /clang:-flto /clang:-fuse-ld=lld-link"
  $Env:CC = "clang-cl"
  $Env:AR = "llvm-lib"
  cargo --config 'target.x86_64-pc-windows-msvc.linker = "lld-link"' build -Z build-std=std,panic_abort --release --locked --target x86_64-pc-windows-msvc
  Copy-Item target/x86_64-pc-windows-msvc/release/TinyWiiBackupManager.exe .

build-windows-x86_64-v2:
  #!pwsh
  $ErrorActionPreference = "Stop"
  $Env:RUSTC_BOOTSTRAP = "1"
  $Env:RUSTFLAGS = "-C target-feature=+crt-static -C target-cpu=x86-64-v2 -C linker-plugin-lto"
  $Env:CFLAGS = "/clang:-O3 /clang:-march=x86-64-v2 /clang:-flto /clang:-fuse-ld=lld-link"
  $Env:CC = "clang-cl"
  $Env:AR = "llvm-lib"
  cargo --config 'target.x86_64-pc-windows-msvc.linker = "lld-link"' build -Z build-std=std,panic_abort --release --locked --target x86_64-pc-windows-msvc
  Copy-Item target/x86_64-pc-windows-msvc/release/TinyWiiBackupManager.exe .

build-windows-x86_64-v3:
  #!pwsh
  $ErrorActionPreference = "Stop"
  $Env:RUSTC_BOOTSTRAP = "1"
  $Env:RUSTFLAGS = "-C target-feature=+crt-static -C target-cpu=x86-64-v3 -C linker-plugin-lto"
  $Env:CFLAGS = "/clang:-O3 /clang:-march=x86-64-v3 /clang:-flto /clang:-fuse-ld=lld-link"
  $Env:CC = "clang-cl"
  $Env:AR = "llvm-lib"
  cargo --config 'target.x86_64-pc-windows-msvc.linker = "lld-link"' build -Z build-std=std,panic_abort --release --locked --target x86_64-pc-windows-msvc
  Copy-Item target/x86_64-pc-windows-msvc/release/TinyWiiBackupManager.exe .

build-windows-arm64:
  #!pwsh
  $ErrorActionPreference = "Stop"
  $Env:RUSTC_BOOTSTRAP = "1"
  $Env:RUSTFLAGS = "-C target-feature=+crt-static -C linker-plugin-lto"
  $Env:CFLAGS = "/clang:-O3 /clang:-flto /clang:-fuse-ld=lld-link"
  $Env:CC = "clang-cl"
  $Env:AR = "llvm-lib"
  cargo --config 'target.aarch64-pc-windows-msvc.linker = "lld-link"' build -Z build-std=std,panic_abort --release --locked --target x86_64-pc-windows-msvc
  Copy-Item target/aarch64-pc-windows-msvc/release/TinyWiiBackupManager.exe .

build-windows-x86:
  #!pwsh
  $ErrorActionPreference = "Stop"
  $Env:RUSTC_BOOTSTRAP = "1"
  $Env:RUSTFLAGS = "-C target-feature=+crt-static"
  $Env:CFLAGS = "/clang:-O3"
  $Env:CC = "clang-cl"
  cargo build -Z build-std=std,panic_abort --release --locked --target i686-pc-windows-msvc
  Copy-Item target/i686-pc-windows-msvc/release/TinyWiiBackupManager.exe .

build-windows-legacy-x86_64:
  #!pwsh
  $ErrorActionPreference = "Stop"
  $Env:RUSTC_BOOTSTRAP = "1"
  $Env:RUSTFLAGS = "-C target-feature=+crt-static"
  $Env:CFLAGS = "/clang:-O3"
  $Env:CC = "clang-cl"
  cargo build -Z build-std=std,panic_abort --release --locked --target x86_64-win7-windows-msvc
  Copy-Item target/x86_64-win7-windows-msvc/release/TinyWiiBackupManager.exe .

build-windows-legacy-x86:
  #!pwsh
  $ErrorActionPreference = "Stop"
  $Env:RUSTC_BOOTSTRAP = "1"
  $Env:RUSTFLAGS = "-C target-feature=+crt-static"
  $Env:CFLAGS = "/clang:-O3"
  $Env:CC = "clang-cl"
  cargo build -Z build-std=std,panic_abort --release --locked --target i686-win7-windows-msvc
  Copy-Item target/i686-win7-windows-msvc/release/TinyWiiBackupManager.exe .

package-windows-zip version-name platform arch:
  #!pwsh
  $ErrorActionPreference = "Stop"
  New-Item -Path "dist" -ItemType Directory
  7z a -tzip -mx=9 "dist/TinyWiiBackupManager-{{ version-name }}-{{ platform }}-{{ arch }}.zip" TinyWiiBackupManager.exe


build-macos-arm64:
  #!/bin/bash
  set -euo pipefail
  export RUSTC_BOOTSTRAP=1
  export RUSTFLAGS="-C link-arg=-mmacosx-version-min=11.0"
  export CFLAGS="-O3 -flto"
  cargo build -Z build-std=std,panic_abort --release --locked --target aarch64-apple-darwin
  cp target/aarch64-apple-darwin/release/TinyWiiBackupManager .

build-macos-x86_64:
  #!/bin/bash
  set -euo pipefail
  export RUSTC_BOOTSTRAP=1
  export RUSTFLAGS="-C link-arg=-mmacosx-version-min=10.13"
  export CFLAGS="-O3 -flto"
  cargo build -Z build-std=std,panic_abort --release --locked --target x86_64-apple-darwin
  cp target/x86_64-apple-darwin/release/TinyWiiBackupManager .

package-macos-app:
  #!/bin/bash
  set -euo pipefail
  mkdir -p TinyWiiBackupManager.app/Contents/MacOS TinyWiiBackupManager.app/Contents/Resources
  cp TinyWiiBackupManager TinyWiiBackupManager.app/Contents/MacOS/TinyWiiBackupManager
  cp package/macos/TinyWiiBackupManager.icns TinyWiiBackupManager.app/Contents/Resources/TinyWiiBackupManager.icns
  cp package/macos/Info.plist TinyWiiBackupManager.app/Contents/Info.plist
  /usr/libexec/PlistBuddy -c "Add :CFBundleShortVersionString string {{ version }}" TinyWiiBackupManager.app/Contents/Info.plist

zip-macos-app version-name arch:
  #!/bin/bash
  set -euo pipefail
  mkdir out
  ditto -c -k \
    --sequesterRsrc \
    --keepParent \
    --zlibCompressionLevel 9 \
    TinyWiiBackupManager.app \
    "out/TinyWiiBackupManager-{{ version-name }}-macos-{{ arch }}.zip"
