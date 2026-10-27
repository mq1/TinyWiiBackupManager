[windows]
set shell := ["pwsh", "-NoLogo", "-NoProfile", "-Command"]

export RUSTC_BOOTSTRAP := "1"
export VERSION := replace_regex(read("Cargo.toml"), '(?s).*?\nversion\s*=\s*"([^"]+)".*', "$1")
version_name := env("VERSION_NAME", "dev")


default:
    @just --list


# Linux
# -----

[linux]
_build-linux target arch:
    cargo build-dist-appimage --target {{target}}
    mkdir -p out
    cp target/{{target}}/release/UltimateGameBackupManager out/UltimateGameBackupManager-linux-{{arch}}

[linux]
build-linux-x86_64: (_build-linux "x86_64-unknown-linux-gnu" "x86_64")
[linux]
build-linux-aarch64: (_build-linux "aarch64-unknown-linux-gnu" "aarch64")
[linux]
build-linux-i686: (_build-linux "i686-unknown-linux-gnu" "i686")

[linux]
_package-linux $ARCH:
    cp -rP package/linux/AppDir UltimateGameBackupManager.AppDir
    install -Dm0755 out/UltimateGameBackupManager-linux-{{ARCH}} UltimateGameBackupManager.AppDir/usr/bin/UltimateGameBackupManager

    mkdir -p dist && cd dist && appimagetool \
        --comp zstd --mksquashfs-opt -Xcompression-level --mksquashfs-opt 19 \
        -u "gh-releases-zsync|petter1|UltimateGameBackupManager|latest|*{{ARCH}}.AppImage.zsync" \
        ../UltimateGameBackupManager.AppDir \
        "UltimateGameBackupManager-{{version_name}}-linux-{{ARCH}}.AppImage"

[linux]
package-linux-x86_64: (_package-linux "x86_64")
[linux]
package-linux-aarch64: (_package-linux "aarch64")
[linux]
package-linux-i686: (_package-linux "i686")


# Windows
# -------

[windows]
_build-windows target arch variant:
    cargo build-dist-windows-{{variant}} --target {{target}}
    New-Item -ItemType Directory -Force -Path out | Out-Null
    Copy-Item -Path target\{{target}}\release\UltimateGameBackupManager.exe -Destination out\UltimateGameBackupManager-windows-{{arch}}.exe

[windows]
build-windows-x64: (_build-windows "x86_64-win7-windows-msvc" "x64" "modern")
[windows]
build-windows-arm64: (_build-windows "aarch64-pc-windows-msvc" "arm64" "modern")
[windows]
build-windows-x86: (_build-windows "i686-win7-windows-msvc" "x86" "legacy")

# just a copy
[windows]
_package-windows arch:
    New-Item -ItemType Directory -Force -Path dist | Out-Null
    Copy-Item -Path "out\UltimateGameBackupManager-windows-{{arch}}.exe" -Destination "dist\UltimateGameBackupManager-{{version_name}}-windows-{{arch}}.exe"

[windows]
package-windows-x64: (_package-windows "x64")
[windows]
package-windows-arm64: (_package-windows "arm64")
[windows]
package-windows-x86: (_package-windows "x86")

# macOS
# -----

[macos]
_build-macos target arch $MACOSX_DEPLOYMENT_TARGET:
    cargo build-dist-macos --target {{target}}
    mkdir -p out
    cp target/{{target}}/release/UltimateGameBackupManager out/UltimateGameBackupManager-macos-{{arch}}

[macos]
build-macos-x86_64: (_build-macos "x86_64-apple-darwin" "x86_64" "10.12")
[macos]
build-macos-arm64: (_build-macos "aarch64-apple-darwin" "arm64" "11.0")

# requires node + appdmg (npm i -g appdmg)
[macos]
package-macos-universal:
    # Create the .app bundle
    cp -r package/macos/App UltimateGameBackupManager.app
    mkdir -p UltimateGameBackupManager.app/Contents/MacOS
    /usr/libexec/PlistBuddy -c "Add :CFBundleShortVersionString string {{VERSION}}" UltimateGameBackupManager.app/Contents/Info.plist

    # Build the fat binary
    lipo -create out/UltimateGameBackupManager-macos-x86_64 out/UltimateGameBackupManager-macos-arm64 -output UltimateGameBackupManager.app/Contents/MacOS/UltimateGameBackupManager
    chmod +x UltimateGameBackupManager.app/Contents/MacOS/UltimateGameBackupManager

    # Sign the .app bundle
    codesign --force --deep -s - UltimateGameBackupManager.app

    # Create the DMG
    mkdir -p dist
    appdmg package/macos/appdmg.json "dist/UltimateGameBackupManager-{{version_name}}-macos-universal.dmg"


# Utils
# -----

print-current-version:
    @echo {{VERSION}}


