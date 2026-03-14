# SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
# SPDX-License-Identifier: GPL-3.0-only

set shell := ["bash", "-uc"]
set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

export RUSTC_BOOTSTRAP := "1"


# ===========
# LINUX BUILD
# ===========

build-linux-x86_64:
  CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_RUSTFLAGS="-C linker-plugin-lto -C link-arg=-fuse-ld=lld-21 -C link-arg=--target=x86_64-unknown-linux-musl" \
    CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER="clang-21" \
    CC_x86_64_unknown_linux_musl="clang-21" \
    AR_x86_64_unknown_linux_musl="llvm-ar-21" \
    CFLAGS_x86_64_unknown_linux_musl="-O3 -flto" \
    cargo build -Z build-std=std,panic_abort --release --locked --target x86_64-unknown-linux-musl --features mimalloc
  cp target/x86_64-unknown-linux-musl/release/TinyWiiBackupManager .

build-linux-x86_64-v2:
  CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_RUSTFLAGS="-C linker-plugin-lto -C link-arg=-fuse-ld=lld-21 -C link-arg=--target=x86_64-unknown-linux-musl -C target-cpu=x86-64-v2" \
    CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER="clang-21" \
    CC_x86_64_unknown_linux_musl="clang-21" \
    AR_x86_64_unknown_linux_musl="llvm-ar-21" \
    CFLAGS_x86_64_unknown_linux_musl="-O3 -flto -march=x86-64-v2" \
    cargo build -Z build-std=std,panic_abort --release --locked --target x86_64-unknown-linux-musl --features mimalloc
  cp target/x86_64-unknown-linux-musl/release/TinyWiiBackupManager .

build-linux-x86_64-v3:
  CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_RUSTFLAGS="-C linker-plugin-lto -C link-arg=-fuse-ld=lld-21 -C link-arg=--target=x86_64-unknown-linux-musl -C target-cpu=x86-64-v3" \
    CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER="clang-21" \
    CC_x86_64_unknown_linux_musl="clang-21" \
    AR_x86_64_unknown_linux_musl="llvm-ar-21" \
    CFLAGS_x86_64_unknown_linux_musl="-O3 -flto -march=x86-64-v3" \
    cargo build -Z build-std=std,panic_abort --release --locked --target x86_64-unknown-linux-musl --features mimalloc
  cp target/x86_64-unknown-linux-musl/release/TinyWiiBackupManager .

build-linux-arm64:
  CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_RUSTFLAGS="-C linker-plugin-lto -C link-arg=-fuse-ld=lld-21 -C link-arg=--target=aarch64-unknown-linux-musl" \
    CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER="clang-21" \
    CC_aarch64_unknown_linux_musl="clang-21" \
    AR_aarch64_unknown_linux_musl="llvm-ar-21" \
    CFLAGS_aarch64_unknown_linux_musl="-O3 -flto" \
    cargo build -Z build-std=std,panic_abort --release --locked --target aarch64-unknown-linux-musl --features mimalloc
  cp target/aarch64-unknown-linux-musl/release/TinyWiiBackupManager .

build-linux-x86:
  CARGO_TARGET_I686_UNKNOWN_LINUX_MUSL_RUSTFLAGS="-C linker-plugin-lto -C link-arg=-fuse-ld=lld-21 -C link-arg=--target=i686-unknown-linux-musl" \
    CARGO_TARGET_I686_UNKNOWN_LINUX_MUSL_LINKER="clang-21" \
    CC_i686_unknown_linux_musl="clang-21" \
    AR_i686_unknown_linux_musl="llvm-ar-21" \
    CFLAGS_i686_unknown_linux_musl="-O3 -flto" \
    cargo build -Z build-std=std,panic_abort --release --locked --target i686-unknown-linux-musl
  cp target/i686-unknown-linux-musl/release/TinyWiiBackupManager .

build-linux-armhf:
  CARGO_TARGET_ARMV7_UNKNOWN_LINUX_MUSLEABIHF_RUSTFLAGS="-C linker-plugin-lto -C link-arg=-fuse-ld=lld-21 -C link-arg=--target=armv7-unknown-linux-musleabihf" \
    CARGO_TARGET_ARMV7_UNKNOWN_LINUX_MUSLEABIHF_LINKER="clang-21" \
    CC_armv7_unknown_linux_musleabihf="clang-21" \
    AR_armv7_unknown_linux_musleabihf="llvm-ar-21" \
    CFLAGS_armv7_unknown_linux_musleabihf="-O3 -flto" \
    cargo build -Z build-std=std,panic_abort --release --locked --target armv7-unknown-linux-musleabihf
  cp target/armv7-unknown-linux-musleabihf/release/TinyWiiBackupManager .

package-linux-tarball version-name arch:
  mkdir -p dist
  tar \
    -I 'gzip -9' \
    --owner=0 \
    --group=0 \
    --mode=0755 \
    -cvf "dist/TinyWiiBackupManager-{{ version-name }}-linux-{{ arch }}.tar.gz" \
    TinyWiiBackupManager

package-linux-appimage $VERSION_NAME arch appimagetool appimage-arch:
  cp -r package/linux/AppDir TinyWiiBackupManager.AppDir
  install -Dm0755 TinyWiiBackupManager TinyWiiBackupManager.AppDir/usr/bin/TinyWiiBackupManager
  mkdir -p dist
  VERSION="${VERSION_NAME#v}" \
    ARCH="{{ appimage-arch }}" \
    {{ appimagetool }} \
    -u "gh-releases-zsync|mq1|TinyWiiBackupManager|latest|*{{ arch }}.AppImage.zsync" \
    TinyWiiBackupManager.AppDir \
    "dist/TinyWiiBackupManager-${VERSION_NAME}-linux-{{ arch }}.AppImage"
  cp *.zsync dist/


# =============
# WINDOWS BUILD
# =============

build-windows-x86_64:
  $Env:CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS = "-C target-feature=+crt-static -C linker-plugin-lto -L native=VC-LTL-Binary\\TargetPlatform\\10.0.19041.0\\lib\\x64"; \
    $Env:CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER = "lld-link"; \
    $Env:CC_x86_64_pc_windows_msvc = "clang-cl"; \
    $Env:AR_x86_64_pc_windows_msvc = "llvm-lib"; \
    $Env:CFLAGS_x86_64_pc_windows_msvc = "/clang:-O3 /clang:-flto /clang:-fuse-ld=lld-link"; \
    cargo build -Z build-std=std,panic_abort --release --locked --target x86_64-pc-windows-msvc
  Copy-Item target\x86_64-pc-windows-msvc\release\TinyWiiBackupManager.exe .

build-windows-x86_64-v2:
  $Env:CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS = "-C target-cpu=x86-64-v2 -C target-feature=+crt-static -C linker-plugin-lto -L native=VC-LTL-Binary\\TargetPlatform\\10.0.19041.0\\lib\\x64"; \
    $Env:CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER = "lld-link"; \
    $Env:CC_x86_64_pc_windows_msvc = "clang-cl"; \
    $Env:AR_x86_64_pc_windows_msvc = "llvm-lib"; \
    $Env:CFLAGS_x86_64_pc_windows_msvc = "/clang:-march=x86-64-v2 /clang:-O3 /clang:-flto /clang:-fuse-ld=lld-link"; \
    cargo build -Z build-std=std,panic_abort --release --locked --target x86_64-pc-windows-msvc
  Copy-Item target\x86_64-pc-windows-msvc\release\TinyWiiBackupManager.exe .

build-windows-x86_64-v3:
  $Env:CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS = "-C target-cpu=x86-64-v3 -C target-feature=+crt-static -C linker-plugin-lto -L native=VC-LTL-Binary\\TargetPlatform\\10.0.19041.0\\lib\\x64"; \
    $Env:CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER = "lld-link"; \
    $Env:CC_x86_64_pc_windows_msvc = "clang-cl"; \
    $Env:AR_x86_64_pc_windows_msvc = "llvm-lib"; \
    $Env:CFLAGS_x86_64_pc_windows_msvc = "/clang:-march=x86-64-v3 /clang:-O3 /clang:-flto /clang:-fuse-ld=lld-link"; \
    cargo build -Z build-std=std,panic_abort --release --locked --target x86_64-pc-windows-msvc
  Copy-Item target\x86_64-pc-windows-msvc\release\TinyWiiBackupManager.exe .

build-windows-arm64:
  $Env:CARGO_TARGET_AARCH64_PC_WINDOWS_MSVC_RUSTFLAGS = "-C target-feature=+crt-static -C linker-plugin-lto -L native=VC-LTL-Binary\\TargetPlatform\\10.0.19041.0\\lib\\ARM64"; \
    $Env:CARGO_TARGET_AARCH64_PC_WINDOWS_MSVC_LINKER = "lld-link"; \
    $Env:CC_aarch64_pc_windows_msvc = "clang-cl"; \
    $Env:AR_aarch64_pc_windows_msvc = "llvm-lib"; \
    $Env:CFLAGS_aarch64_pc_windows_msvc = "/clang:-O3 /clang:-flto /clang:-fuse-ld=lld-link"; \
    cargo build -Z build-std=std,panic_abort --release --locked --target aarch64-pc-windows-msvc
  Copy-Item target\aarch64-pc-windows-msvc\release\TinyWiiBackupManager.exe .

build-windows-x86:
  $Env:CARGO_TARGET_I686_PC_WINDOWS_MSVC_RUSTFLAGS = "-C target-feature=+crt-static -L native=VC-LTL-Binary\\TargetPlatform\\10.0.19041.0\\lib\\Win32"; \
    $Env:CC_i686_pc_windows_msvc = "clang-cl"; \
    $Env:CFLAGS_i686_pc_windows_msvc = "/clang:-O3"; \
    cargo build -Z build-std=std,panic_abort --release --locked --target i686-pc-windows-msvc
  Copy-Item target\i686-pc-windows-msvc\release\TinyWiiBackupManager.exe .

build-windows-legacy-x86_64:
  $Env:CARGO_TARGET_X86_64_WIN7_WINDOWS_MSVC_RUSTFLAGS = "-C target-feature=+crt-static -l dylib=ole32 -L native=VC-LTL-Binary\\TargetPlatform\\5.2.3790.0\\lib\\x64 -C link-arg=YY-Thunks-Objs\\objs\\x64\\YY_Thunks_for_WinXP.obj -C link-arg=/SUBSYSTEM:WINDOWS,5.2"; \
    $Env:CC_x86_64_win7_windows_msvc = "clang-cl"; \
    $Env:CFLAGS_x86_64_win7_windows_msvc = "/clang:-O3"; \
    cargo build -Z build-std=std,panic_abort --release --locked --target x86_64-win7-windows-msvc
  Copy-Item target\x86_64-win7-windows-msvc\release\TinyWiiBackupManager.exe .

build-windows-legacy-x86:
  $Env:CARGO_TARGET_I686_WIN7_WINDOWS_MSVC_RUSTFLAGS = "-C target-feature=+crt-static -l dylib=ole32 -L native=VC-LTL-Binary\\TargetPlatform\\5.1.2600.0\\lib\\Win32 -C link-arg=YY-Thunks-Objs\\objs\\x86\\YY_Thunks_for_WinXP.obj -C link-arg=/SUBSYSTEM:WINDOWS,5.1"; \
    $Env:CC_i686_win7_windows_msvc = "clang-cl"; \
    $Env:CFLAGS_i686_win7_windows_msvc = "/clang:-O3"; \
    cargo build -Z build-std=std,panic_abort --release --locked --target i686-win7-windows-msvc
  Copy-Item target\i686-win7-windows-msvc\release\TinyWiiBackupManager.exe .

package-windows-zip version-name platform arch:
  New-Item -Path "dist" -ItemType Directory
  7z a -tzip -mx=9 "dist\TinyWiiBackupManager-{{ version-name }}-{{ platform }}-{{ arch }}.zip" TinyWiiBackupManager.exe


# ===========
# MACOS BUILD
# ===========

build-macos-arm64:
  MACOSX_DEPLOYMENT_TARGET="11.0" \
    CARGO_TARGET_AARCH64_APPLE_DARWIN_RUSTFLAGS="-C link-arg=-mmacosx-version-min=11.0" \
    CC_aarch64_apple_darwin="/opt/homebrew/opt/llvm@21/bin/clang" \
    CFLAGS_aarch64_apple_darwin="-O3" \
    cargo build -Z build-std=std,panic_abort --release --locked --target aarch64-apple-darwin
  cp target/aarch64-apple-darwin/release/TinyWiiBackupManager .

build-macos-x86_64:
  MACOSX_DEPLOYMENT_TARGET="10.13" \
    CARGO_TARGET_X86_64_APPLE_DARWIN_RUSTFLAGS="-C link-arg=-mmacosx-version-min=10.13" \
    CC_aarch64_apple_darwin="/opt/homebrew/opt/llvm@21/bin/clang" \
    CFLAGS_x86_64_apple_darwin="-O3" \
    cargo build -Z build-std=std,panic_abort --release --locked --target x86_64-apple-darwin
  cp target/x86_64-apple-darwin/release/TinyWiiBackupManager .

package-macos-app $VERSION_NAME:
  mkdir -p TinyWiiBackupManager.app/Contents/MacOS TinyWiiBackupManager.app/Contents/Resources
  cp TinyWiiBackupManager TinyWiiBackupManager.app/Contents/MacOS/TinyWiiBackupManager
  cp package/macos/TinyWiiBackupManager.icns TinyWiiBackupManager.app/Contents/Resources/TinyWiiBackupManager.icns
  cp package/macos/Info.plist TinyWiiBackupManager.app/Contents/Info.plist
  /usr/libexec/PlistBuddy -c "Add :CFBundleShortVersionString string ${VERSION_NAME#v}" TinyWiiBackupManager.app/Contents/Info.plist

zip-macos-app version-name arch:
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

[script("python3")]
print-changes version-name:
  with open("CHANGELOG.md") as f:
      grab=False
      for line in f:
          if line.startswith(f"## [{{ version-name }}]"): grab=True; continue
          if grab and line.startswith("## ["): break
          if grab: print(line, end="")

  print(f"""<br>

  <table>
    <tr>
      <td width="9999px"><strong>:arrow_down: Recommended downloads</strong></td>
    </tr>
    <tr>
      <td>
        :window: <a href="https://github.com/mq1/TinyWiiBackupManager/releases/download/{{ version-name }}/TinyWiiBackupManager-{{ version-name }}-windows-x86_64.zip">Windows x64 Standalone</a><br>
        :apple: <a href="https://github.com/mq1/TinyWiiBackupManager/releases/download/{{ version-name }}/TinyWiiBackupManager-{{ version-name }}-macos-universal.dmg">macOS Universal Binary</a><br>
        :penguin: <a href="https://github.com/mq1/TinyWiiBackupManager/releases/download/{{ version-name }}/TinyWiiBackupManager-{{ version-name }}-linux-x86_64.AppImage">Linux x86_64 AppImage</a>
      </td>
    </tr>
  </table>""")

[script("python3")]
print-scoop-manifest version-name:
  import urllib.request, json

  manifest = {
      "$schema": "https://raw.githubusercontent.com/ScoopInstaller/Scoop/master/schema.json",
      "version": "{{ version-name }}".removeprefix("v"),
      "description": "A tiny game backup and homebrew app manager for the Wii",
      "homepage": "https://github.com/mq1/TinyWiiBackupManager",
      "license": "GPL-3.0-only",
      "shortcuts": [["TinyWiiBackupManager.exe", "TinyWiiBackupManager"]],
      "architecture": {
          "64bit": { "url": "https://github.com/mq1/TinyWiiBackupManager/releases/download/{{ version-name }}/TinyWiiBackupManager-{{ version-name }}-windows-x86_64.zip" },
          "arm64": { "url": "https://github.com/mq1/TinyWiiBackupManager/releases/download/{{ version-name }}/TinyWiiBackupManager-{{ version-name }}-windows-arm64.zip" },
          "32bit": { "url": "https://github.com/mq1/TinyWiiBackupManager/releases/download/{{ version-name }}/TinyWiiBackupManager-{{ version-name }}-windows-x86.zip" }
      }
  }

  with urllib.request.urlopen("https://api.github.com/repos/mq1/TinyWiiBackupManager/releases/tags/{{ version-name }}") as response:
      data = json.load(response)

  for asset in data["assets"]:
      if asset["name"] == "TinyWiiBackupManager-{{ version-name }}-windows-x86_64.zip":
          manifest["architecture"]["64bit"]["hash"] = asset["digest"].removeprefix("sha256:")
      elif asset["name"] == "TinyWiiBackupManager-{{ version-name }}-windows-arm64.zip":
          manifest["architecture"]["arm64"]["hash"] = asset["digest"].removeprefix("sha256:")
      elif asset["name"] == "TinyWiiBackupManager-{{ version-name }}-windows-x86.zip":
          manifest["architecture"]["32bit"]["hash"] = asset["digest"].removeprefix("sha256:")

  print(json.dumps(manifest, indent=2))

[script("python3")]
update-winget version-name:
  import subprocess

  subprocess.check_output([
      "komac",
      "update",
      "mq1.TinyWiiBackupManager",
      "--version",
      "{{ version-name }}".removeprefix("v"),
      "--urls",
      "https://github.com/mq1/TinyWiiBackupManager/releases/download/{{ version-name }}/TinyWiiBackupManager-{{ version-name }}-windows-x86_64.zip",
      "https://github.com/mq1/TinyWiiBackupManager/releases/download/{{ version-name }}/TinyWiiBackupManager-{{ version-name }}-windows-arm64.zip",
      "https://github.com/mq1/TinyWiiBackupManager/releases/download/{{ version-name }}/TinyWiiBackupManager-{{ version-name }}-windows-x86.zip",
      "--submit"
  ])

[script("bash")]
make-icons:
  TARGET_RESOLUTIONS=("16x16" "32x32" "48x48" "64x64" "128x128" "256x256" "512x512")
  MAGICK_ARGS="-strip -colors 8 -dither None"

  # Common
  magick assets/TinyWiiBackupManager-1024x1024.png ${MAGICK_ARGS} -resize 256x256 assets/TinyWiiBackupManager-256x256.png
  oxipng -sao6 assets/TinyWiiBackupManager-256x256.png

  # Linux
  rm -rf package/linux/usr/share/icons
  for res in "${TARGET_RESOLUTIONS[@]}"; do
    mkdir -p package/linux/usr/share/icons/hicolor/${res}/apps
    magick assets/TinyWiiBackupManager-1024x1024.png ${MAGICK_ARGS} -resize ${res} package/linux/usr/share/icons/hicolor/${res}/apps/it.mq1.TinyWiiBackupManager.png
    oxipng -sao6 package/linux/usr/share/icons/hicolor/${res}/apps/it.mq1.TinyWiiBackupManager.png
  done

  # Windows
  rm -f package/windows/icon.ico package/windows/TinyWiiBackupManager-64x64.png
  magick assets/TinyWiiBackupManager-1024x1024.png ${MAGICK_ARGS} -define icon:auto-resize=16,24,32,48,256 package/windows/icon.ico
  magick assets/TinyWiiBackupManager-1024x1024.png ${MAGICK_ARGS} -resize 64x64 package/windows/TinyWiiBackupManager-64x64.png
  oxipng -sao6 package/windows/TinyWiiBackupManager-64x64.png

  # macOS
  rm -f package/macos/TinyWiiBackupManager.icns
  rm -rf package/macos/TinyWiiBackupManager.iconset
  mkdir package/macos/TinyWiiBackupManager.iconset
  magick assets/TinyWiiBackupManager-1024x1024.png ${MAGICK_ARGS} -resize 16x16 package/macos/TinyWiiBackupManager.iconset/icon_16x16.png
  oxipng -sao6 package/macos/TinyWiiBackupManager.iconset/icon_16x16.png
  magick assets/TinyWiiBackupManager-1024x1024.png ${MAGICK_ARGS} -resize 32x32 package/macos/TinyWiiBackupManager.iconset/icon_16x16@2x.png
  oxipng -sao6 package/macos/TinyWiiBackupManager.iconset/icon_16x16@2x.png
  magick assets/TinyWiiBackupManager-1024x1024.png ${MAGICK_ARGS} -resize 64x64 package/macos/TinyWiiBackupManager.iconset/icon_32x32@2x.png
  oxipng -sao6 package/macos/TinyWiiBackupManager.iconset/icon_32x32@2x.png
  magick assets/TinyWiiBackupManager-1024x1024.png ${MAGICK_ARGS} -resize 128x128 package/macos/TinyWiiBackupManager.iconset/icon_128x128.png
  oxipng -sao6 package/macos/TinyWiiBackupManager.iconset/icon_128x128.png
  magick assets/TinyWiiBackupManager-1024x1024.png ${MAGICK_ARGS} -resize 256x256 package/macos/TinyWiiBackupManager.iconset/icon_128x128@2x.png
  oxipng -sao6 package/macos/TinyWiiBackupManager.iconset/icon_128x128@2x.png
  magick assets/TinyWiiBackupManager-1024x1024.png ${MAGICK_ARGS} -resize 256x256 package/macos/TinyWiiBackupManager.iconset/icon_256x256.png
  oxipng -sao6 package/macos/TinyWiiBackupManager.iconset/icon_256x256.png
  magick assets/TinyWiiBackupManager-1024x1024.png ${MAGICK_ARGS} -resize 512x512 package/macos/TinyWiiBackupManager.iconset/icon_256x256@2x.png
  oxipng -sao6 package/macos/TinyWiiBackupManager.iconset/icon_256x256@2x.png
  magick assets/TinyWiiBackupManager-1024x1024.png ${MAGICK_ARGS} -resize 512x512 package/macos/TinyWiiBackupManager.iconset/icon_512x512.png
  oxipng -sao6 package/macos/TinyWiiBackupManager.iconset/icon_512x512.png
  magick assets/TinyWiiBackupManager-1024x1024.png ${MAGICK_ARGS} -resize 1024x1024 package/macos/TinyWiiBackupManager.iconset/icon_512x512@2x.png
  oxipng -sao6 package/macos/TinyWiiBackupManager.iconset/icon_512x512@2x.png
  iconutil -c icns package/macos/TinyWiiBackupManager.iconset

