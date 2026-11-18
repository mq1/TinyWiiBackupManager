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
    cp target/{{target}}/release/TinyWiiBackupManager out/TinyWiiBackupManager-linux-{{arch}}

[linux]
build-linux-x86_64: (_build-linux "x86_64-unknown-linux-gnu" "x86_64")
[linux]
build-linux-aarch64: (_build-linux "aarch64-unknown-linux-gnu" "aarch64")
[linux]
build-linux-i686: (_build-linux "i686-unknown-linux-gnu" "i686")

[linux]
_package-linux $ARCH:
    cp -rP package/linux/AppDir TinyWiiBackupManager.AppDir
    install -Dm0755 out/TinyWiiBackupManager-linux-{{ARCH}} TinyWiiBackupManager.AppDir/usr/bin/TinyWiiBackupManager

    mkdir -p dist && cd dist && appimagetool \
        --comp zstd --mksquashfs-opt -Xcompression-level --mksquashfs-opt 19 \
        -u "gh-releases-zsync|mq1|TinyWiiBackupManager|latest|*{{ARCH}}.AppImage.zsync" \
        ../TinyWiiBackupManager.AppDir \
        "TinyWiiBackupManager-{{version_name}}-linux-{{ARCH}}.AppImage"

[linux]
package-linux-x86_64: (_package-linux "x86_64")
[linux]
package-linux-aarch64: (_package-linux "aarch64")
[linux]
package-linux-i686: (_package-linux "i686")


# Windows
# -------

[windows]
_build-windows target arch:
    cargo build-dist-windows --target {{target}}
    New-Item -ItemType Directory -Force -Path out | Out-Null
    Copy-Item -Path target\{{target}}\release\TinyWiiBackupManager.exe -Destination out\TinyWiiBackupManager-windows-{{arch}}.exe

[windows]
build-windows-x64: (_build-windows "x86_64-win7-windows-msvc" "x64")
[windows]
build-windows-arm64: (_build-windows "aarch64-pc-windows-msvc" "arm64")
[windows]
build-windows-x86: (_build-windows "i686-win7-windows-msvc" "x86")

# just a copy
[windows]
_package-windows arch:
    New-Item -ItemType Directory -Force -Path dist | Out-Null
    Copy-Item -Path "out\TinyWiiBackupManager-windows-{{arch}}.exe" -Destination "dist\TinyWiiBackupManager-{{version_name}}-windows-{{arch}}.exe"

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
    cp target/{{target}}/release/TinyWiiBackupManager out/TinyWiiBackupManager-macos-{{arch}}

[macos]
build-macos-x86_64: (_build-macos "x86_64-apple-darwin" "x86_64" "10.12")
[macos]
build-macos-arm64: (_build-macos "aarch64-apple-darwin" "arm64" "11.0")

# requires node + appdmg (npm i -g appdmg)
[macos]
package-macos-universal:
    # Create the .app bundle
    cp -r package/macos/App TinyWiiBackupManager.app
    mkdir -p TinyWiiBackupManager.app/Contents/MacOS
    /usr/libexec/PlistBuddy -c "Add :CFBundleShortVersionString string {{VERSION}}" TinyWiiBackupManager.app/Contents/Info.plist

    # Build the fat binary
    lipo -create out/TinyWiiBackupManager-macos-x86_64 out/TinyWiiBackupManager-macos-arm64 -output TinyWiiBackupManager.app/Contents/MacOS/TinyWiiBackupManager
    chmod +x TinyWiiBackupManager.app/Contents/MacOS/TinyWiiBackupManager

    # Sign the .app bundle
    codesign --force --deep -s - TinyWiiBackupManager.app

    # Create the DMG
    mkdir -p dist
    appdmg package/macos/appdmg.json "dist/TinyWiiBackupManager-{{version_name}}-macos-universal.dmg"


# Utils
# -----

print-current-version:
    @echo {{VERSION}}

# requires komac
submit-to-winget:
    komac update mq1.TinyWiiBackupManager \
        --version {{VERSION}} \
        --urls \
            https://github.com/mq1/TinyWiiBackupManager/releases/download/v{{VERSION}}/TinyWiiBackupManager-v{{VERSION}}-windows-x64.exe \
            https://github.com/mq1/TinyWiiBackupManager/releases/download/v{{VERSION}}/TinyWiiBackupManager-v{{VERSION}}-windows-x86.exe \
            https://github.com/mq1/TinyWiiBackupManager/releases/download/v{{VERSION}}/TinyWiiBackupManager-v{{VERSION}}-windows-arm64.exe \
        --submit

# requires macos (for iconutil), imagemagick and oxipng
make-icons:
    #!/bin/bash
    TARGET_RESOLUTIONS=("16x16" "32x32" "48x48" "64x64" "128x128" "256x256" "512x512")
    MAGICK_ARGS="-strip -colors 8 -dither None"

    # Common
    magick assets/TinyWiiBackupManager-1024x1024.png ${MAGICK_ARGS} -resize 256x256 crates/gui/assets/TinyWiiBackupManager-256x256.png
    oxipng -sao6 crates/gui/assets/TinyWiiBackupManager-256x256.png

    # Linux
    rm -rf package/linux/usr/share/icons
    for res in "${TARGET_RESOLUTIONS[@]}"; do
      mkdir -p package/linux/usr/share/icons/hicolor/${res}/apps
      magick assets/TinyWiiBackupManager-1024x1024.png ${MAGICK_ARGS} -resize ${res} package/linux/usr/share/icons/hicolor/${res}/apps/it.mq1.TinyWiiBackupManager.png
      oxipng -sao6 package/linux/usr/share/icons/hicolor/${res}/apps/it.mq1.TinyWiiBackupManager.png
    done

    # Windows
    rm -f crates/gui/assets/icon.ico crates/gui/assets/TinyWiiBackupManager-64x64.png
    magick assets/TinyWiiBackupManager-1024x1024.png ${MAGICK_ARGS} -define icon:auto-resize=16,24,32,48,256 crates/gui/assets/icon.ico
    magick assets/TinyWiiBackupManager-1024x1024.png ${MAGICK_ARGS} -resize 64x64 crates/gui/assets/TinyWiiBackupManager-64x64.png
    oxipng -sao6 crates/gui/assets/TinyWiiBackupManager-64x64.png

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
    mv package/macos/TinyWiiBackupManager.icns package/macos/App/Contents/Resources/
