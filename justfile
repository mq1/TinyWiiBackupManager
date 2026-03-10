# SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
# SPDX-License-Identifier: GPL-3.0-only


# ===========
# LINUX BUILD
# ===========

[script("bash")]
build-linux-x86_64:
  #!/bin/bash
  set -euo pipefail
  export RUSTC_BOOTSTRAP=1
  cargo zigbuild -Z build-std=std,panic_abort --release --locked --target x86_64-unknown-linux-gnu.2.17
  cp target/x86_64-unknown-linux-gnu/release/TinyWiiBackupManager .

[script("bash")]
build-linux-x86_64-v2:
  set -euo pipefail
  export RUSTC_BOOTSTRAP=1
  export RUSTFLAGS="-C target-cpu=x86-64-v2"
  export CFLAGS="-mcpu=x86_64_v2"
  cargo zigbuild -Z build-std=std,panic_abort --release --locked --target x86_64-unknown-linux-gnu.2.17
  cp target/x86_64-unknown-linux-gnu/release/TinyWiiBackupManager .

[script("bash")]
build-linux-x86_64-v3:
  set -euo pipefail
  export RUSTC_BOOTSTRAP=1
  export RUSTFLAGS="-C target-cpu=x86-64-v3"
  export CFLAGS="-mcpu=x86_64_v3"
  cargo zigbuild -Z build-std=std,panic_abort --release --locked --target x86_64-unknown-linux-gnu.2.17
  cp target/x86_64-unknown-linux-gnu/release/TinyWiiBackupManager .

[script("bash")]
build-linux-x86:
  set -euo pipefail
  export RUSTC_BOOTSTRAP=1
  cargo zigbuild -Z build-std=std,panic_abort --release --locked --target i686-unknown-linux-gnu.2.17
  cp target/i686-unknown-linux-gnu/release/TinyWiiBackupManager .

[script("bash")]
build-linux-arm64:
  set -euo pipefail
  export RUSTC_BOOTSTRAP=1
  cargo zigbuild -Z build-std=std,panic_abort --release --locked --target aarch64-unknown-linux-gnu.2.17
  cp target/aarch64-unknown-linux-gnu/release/TinyWiiBackupManager .

[script("bash")]
build-linux-armhf:
  set -euo pipefail
  export RUSTC_BOOTSTRAP=1
  cargo zigbuild -Z build-std=std,panic_abort --release --locked --target armv7-unknown-linux-gnueabihf.2.17
  cp target/armv7-unknown-linux-gnueabihf/release/TinyWiiBackupManager .

[script("bash")]
package-linux-tarball version-name arch:
  set -euo pipefail
  mkdir -p dist
  tar \
    -I 'gzip -9' \
    --owner=0 \
    --group=0 \
    --mode=0755 \
    -cvf "dist/TinyWiiBackupManager-{{ version-name }}-linux-{{ arch }}.tar.gz" \
    TinyWiiBackupManager

[script("bash")]
package-linux-appimage version-name arch appimagetool appimage-arch:
  set -euo pipefail
  export VERSION=$(yq '.package.version' Cargo.toml)
  export ARCH={{ appimage-arch }}
  cp -r package/linux TinyWiiBackupManager.AppDir
  install -Dm0755 TinyWiiBackupManager TinyWiiBackupManager.AppDir/usr/bin/TinyWiiBackupManager
  mkdir -p dist
  {{ appimagetool }} \
    -u "gh-releases-zsync|mq1|TinyWiiBackupManager|latest|*{{ arch }}.AppImage.zsync" \
    TinyWiiBackupManager.AppDir \
    dist/TinyWiiBackupManager-{{ version-name }}-linux-{{ arch }}.AppImage
  cp *.zsync dist/


# =============
# WINDOWS BUILD
# =============

[script("pwsh")]
build-windows-x86_64:
  $ErrorActionPreference = "Stop"
  $PSNativeCommandUseErrorActionPreference = $true
  $Env:RUSTC_BOOTSTRAP = "1"
  $Env:RUSTFLAGS = "-C target-feature=+crt-static -C linker-plugin-lto"
  $Env:CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER = "lld-link"
  $Env:CFLAGS = "/clang:-O3 /clang:-flto /clang:-fuse-ld=lld-link"
  $Env:CC = "clang-cl"
  $Env:AR = "llvm-lib"
  cargo build -Z build-std=std,panic_abort --release --locked --target x86_64-pc-windows-msvc
  Copy-Item target/x86_64-pc-windows-msvc/release/TinyWiiBackupManager.exe .

[script("pwsh")]
build-windows-x86_64-v2:
  $ErrorActionPreference = "Stop"
  $PSNativeCommandUseErrorActionPreference = $true
  $Env:RUSTC_BOOTSTRAP = "1"
  $Env:RUSTFLAGS = "-C target-feature=+crt-static -C target-cpu=x86-64-v2 -C linker-plugin-lto"
  $Env:CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER = "lld-link"
  $Env:CFLAGS = "/clang:-O3 /clang:-march=x86-64-v2 /clang:-flto /clang:-fuse-ld=lld-link"
  $Env:CC = "clang-cl"
  $Env:AR = "llvm-lib"
  cargo build -Z build-std=std,panic_abort --release --locked --target x86_64-pc-windows-msvc
  Copy-Item target/x86_64-pc-windows-msvc/release/TinyWiiBackupManager.exe .

[script("pwsh")]
build-windows-x86_64-v3:
  $ErrorActionPreference = "Stop"
  $PSNativeCommandUseErrorActionPreference = $true
  $Env:RUSTC_BOOTSTRAP = "1"
  $Env:RUSTFLAGS = "-C target-feature=+crt-static -C target-cpu=x86-64-v3 -C linker-plugin-lto"
  $Env:CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER = "lld-link"
  $Env:CFLAGS = "/clang:-O3 /clang:-march=x86-64-v3 /clang:-flto /clang:-fuse-ld=lld-link"
  $Env:CC = "clang-cl"
  $Env:AR = "llvm-lib"
  cargo build -Z build-std=std,panic_abort --release --locked --target x86_64-pc-windows-msvc
  Copy-Item target/x86_64-pc-windows-msvc/release/TinyWiiBackupManager.exe .

[script("pwsh")]
build-windows-arm64:
  $ErrorActionPreference = "Stop"
  $PSNativeCommandUseErrorActionPreference = $true
  $Env:RUSTC_BOOTSTRAP = "1"
  $Env:RUSTFLAGS = "-C target-feature=+crt-static -C linker-plugin-lto"
  $Env:CARGO_TARGET_AARCH64_PC_WINDOWS_MSVC_LINKER = "lld-link"
  $Env:CFLAGS = "/clang:-O3 /clang:-flto /clang:-fuse-ld=lld-link"
  $Env:CC = "clang-cl"
  $Env:AR = "llvm-lib"
  cargo build -Z build-std=std,panic_abort --release --locked --target aarch64-pc-windows-msvc
  Copy-Item target/aarch64-pc-windows-msvc/release/TinyWiiBackupManager.exe .

[script("pwsh")]
build-windows-x86:
  $ErrorActionPreference = "Stop"
  $PSNativeCommandUseErrorActionPreference = $true
  $Env:RUSTC_BOOTSTRAP = "1"
  $Env:RUSTFLAGS = "-C target-feature=+crt-static"
  $Env:CFLAGS = "/clang:-O3"
  $Env:CC = "clang-cl"
  cargo build -Z build-std=std,panic_abort --release --locked --target i686-pc-windows-msvc
  Copy-Item target/i686-pc-windows-msvc/release/TinyWiiBackupManager.exe .

[script("pwsh")]
build-windows-legacy-x86_64:
  $ErrorActionPreference = "Stop"
  $PSNativeCommandUseErrorActionPreference = $true
  $Env:RUSTC_BOOTSTRAP = "1"
  $Env:RUSTFLAGS = "-C target-feature=+crt-static"
  $Env:CFLAGS = "/clang:-O3"
  $Env:CC = "clang-cl"
  cargo build -Z build-std=std,panic_abort --release --locked --target x86_64-win7-windows-msvc
  Copy-Item target/x86_64-win7-windows-msvc/release/TinyWiiBackupManager.exe .

[script("pwsh")]
build-windows-legacy-x86:
  $ErrorActionPreference = "Stop"
  $PSNativeCommandUseErrorActionPreference = $true
  $Env:RUSTC_BOOTSTRAP = "1"
  $Env:RUSTFLAGS = "-C target-feature=+crt-static"
  $Env:CFLAGS = "/clang:-O3"
  $Env:CC = "clang-cl"
  cargo build -Z build-std=std,panic_abort --release --locked --target i686-win7-windows-msvc
  Copy-Item target/i686-win7-windows-msvc/release/TinyWiiBackupManager.exe .

[script("pwsh")]
package-windows-zip version-name platform arch:
  $ErrorActionPreference = "Stop"
  $PSNativeCommandUseErrorActionPreference = $true
  New-Item -Path "dist" -ItemType Directory
  7z a -tzip -mx=9 "dist/TinyWiiBackupManager-{{ version-name }}-{{ platform }}-{{ arch }}.zip" TinyWiiBackupManager.exe


# ===========
# MACOS BUILD
# ===========

[script("zsh")]
build-macos-arm64:
  set -euo pipefail
  export RUSTC_BOOTSTRAP=1
  export RUSTFLAGS="-C link-arg=-mmacosx-version-min=11.0"
  export CFLAGS="-O3 -flto"
  cargo build -Z build-std=std,panic_abort --release --locked --target aarch64-apple-darwin
  cp target/aarch64-apple-darwin/release/TinyWiiBackupManager .

[script("zsh")]
build-macos-x86_64:
  set -euo pipefail
  export RUSTC_BOOTSTRAP=1
  export RUSTFLAGS="-C link-arg=-mmacosx-version-min=10.13"
  export CFLAGS="-O3 -flto"
  cargo build -Z build-std=std,panic_abort --release --locked --target x86_64-apple-darwin
  cp target/x86_64-apple-darwin/release/TinyWiiBackupManager .

[script("zsh")]
package-macos-app:
  set -euo pipefail
  mkdir -p TinyWiiBackupManager.app/Contents/MacOS TinyWiiBackupManager.app/Contents/Resources
  cp TinyWiiBackupManager TinyWiiBackupManager.app/Contents/MacOS/TinyWiiBackupManager
  cp package/macos/TinyWiiBackupManager.icns TinyWiiBackupManager.app/Contents/Resources/TinyWiiBackupManager.icns
  cp package/macos/Info.plist TinyWiiBackupManager.app/Contents/Info.plist
  /usr/libexec/PlistBuddy -c "Add :CFBundleShortVersionString string $(yq '.package.version' Cargo.toml)" TinyWiiBackupManager.app/Contents/Info.plist

[script("zsh")]
zip-macos-app version-name arch:
  set -euo pipefail
  mkdir out
  ditto -c -k \
    --sequesterRsrc \
    --keepParent \
    --zlibCompressionLevel 9 \
    TinyWiiBackupManager.app \
    "out/TinyWiiBackupManager-{{ version-name }}-macos-{{ arch }}.zip"


# =================
# RELEASE UTILITIES
# =================

[script("bash")]
print-changes version-name:
  if ! grep -Fq "## [{{ version-name }}]" CHANGELOG.md; then
    exit 1
  fi

  awk "/^## \[{{ version-name }}\]/{f=1;next} /^## \[/{f=0} f" CHANGELOG.md

  cat <<EOF
  <br> 

  <table>
    <tr>
      <td width="9999px"><strong>:arrow_down: Recommended downloads</strong></td>
    </tr>
    <tr>
      <td>
        :window: <a href="https://github.com/mq1/TinyWiiBackupManager/releases/download/{{ version-name }}/TinyWiiBackupManager-{{ version-name }}-windows-x86_64.zip">Windows x64 Standalone</a>
        <br>
        :apple: <a href="https://github.com/mq1/TinyWiiBackupManager/releases/download/{{ version-name }}/TinyWiiBackupManager-{{ version-name }}-macos-universal.dmg">macOS Universal Binary</a>
        <br>
        :penguin: <a href="https://github.com/mq1/TinyWiiBackupManager/releases/download/{{ version-name }}/TinyWiiBackupManager-{{ version-name }}-linux-x86_64.AppImage">Linux x86_64 AppImage</a>
      </td>
    </tr>
  </table>
  EOF
