export RUSTC_BOOTSTRAP := "1"
version_name := env("VERSION_NAME", "dev")

[unix]
export VERSION := `./scripts/current-version.sh`

[windows]
export VERSION := "" # not used for now

default:
    @just --list

# Linux
# -----

[env("CC", "clang")]
[env("CXX", "clang++")]
[env("AR", "llvm-ar")]
[env("RANLIB", "llvm-ranlib")]
[env("CFLAGS", "-O3 -flto")]
[env("CXXFLAGS", "-O3 -flto")]
[env("RUSTFLAGS", "-C linker-plugin-lto -C linker=clang -C link-arg=-fuse-ld=lld")]
_build-linux target arch:
    cargo build -Z build-std=std,panic_abort --release --target {{target}} --no-default-features --features dist-appimage
    mkdir -p out
    cp target/{{target}}/release/TinyWiiBackupManager out/TinyWiiBackupManager-linux-{{arch}}

build-linux-x86_64: (_build-linux "x86_64-unknown-linux-gnu" "x86_64")
build-linux-aarch64: (_build-linux "aarch64-unknown-linux-gnu" "aarch64")
build-linux-i686: (_build-linux "i686-unknown-linux-gnu" "i686")

_package-linux $ARCH:
    cp -rP package/linux/AppDir TinyWiiBackupManager.AppDir
    install -Dm0755 out/TinyWiiBackupManager-linux-{{ARCH}} TinyWiiBackupManager.AppDir/usr/bin/TinyWiiBackupManager

    mkdir -p dist && cd dist && appimagetool \
        --comp zstd --mksquashfs-opt -Xcompression-level --mksquashfs-opt 19 \
        -u "gh-releases-zsync|mq1|TinyWiiBackupManager|latest|*{{ARCH}}.AppImage.zsync" \
        ../TinyWiiBackupManager.AppDir \
        "TinyWiiBackupManager-{{version_name}}-linux-{{ARCH}}.AppImage"

package-linux-x86_64: (_package-linux "x86_64")
package-linux-aarch64: (_package-linux "aarch64")
package-linux-i686: (_package-linux "i686")


# Windows
# -------

[env("CC", "clang-cl")]
[env("CXX", "clang-cl")]
_build-windows target arch features $AR $CFLAGS $CXXFLAGS $RUSTFLAGS:
    cargo build -Z build-std=std,panic_abort --release --target {{target}} --no-default-features --features {{features}}
    New-Item -ItemType Directory -Force -Path out | Out-Null
    Copy-Item -Path target\{{target}}\release\TinyWiiBackupManager.exe -Destination out\TinyWiiBackupManager-windows-{{arch}}.exe

build-windows-x64: (_build-windows "x86_64-win7-windows-msvc" "x64" "dist-windows-modern" "llvm-lib" "/clang:-O3 /clang:-flto /clang:-fuse-ld=lld-link" "/clang:-O3 /clang:-flto /clang:-fuse-ld=lld-link" "-C target-feature=+crt-static -C linker-plugin-lto -C linker=lld-link -C link-arg=/FORCE:MULTIPLE")

build-windows-arm64: (_build-windows "aarch64-pc-windows-msvc" "arm64" "dist-windows-modern" "llvm-lib" "/clang:-O3 /clang:-flto /clang:-fuse-ld=lld-link" "/clang:-O3 /clang:-flto /clang:-fuse-ld=lld-link" "-C target-feature=+crt-static -C linker-plugin-lto -C linker=lld-link")

# requires nasm (needed for aws-lc)
build-windows-x86: (_build-windows "i686-win7-windows-msvc" "x86" "dist-windows-legacy" "" "/clang:-O3" "/clang:-O3" "-C target-feature=+crt-static")

# just a copy
_package-windows arch:
    New-Item -ItemType Directory -Force -Path dist | Out-Null
    Copy-Item -Path "out\TinyWiiBackupManager-windows-{{arch}}.exe" -Destination "dist\TinyWiiBackupManager-{{version_name}}-windows-{{arch}}.exe"

package-windows-x64: (_package-windows "x64")
package-windows-arm64: (_package-windows "arm64")
package-windows-x86: (_package-windows "x86")

# macOS
# -----

[env("CC", "clang")]
[env("CXX", "clang++")]
[env("AR", "llvm-ar")]
[env("CFLAGS", "-O3 -flto")]
[env("CXXFLAGS", "-O3 -flto")]
[env("RUSTFLAGS", "-C linker-plugin-lto -C linker=" + justfile_dir() + "/macos-linker.sh -C link-arg=-fuse-ld=lld")]
_build-macos target arch $MACOSX_DEPLOYMENT_TARGET:
    cargo build -Z build-std=std,panic_abort --release --target {{target}} --no-default-features --features dist-macos
    mkdir -p out
    cp target/{{target}}/release/TinyWiiBackupManager out/TinyWiiBackupManager-macos-{{arch}}

build-macos-x86_64: (_build-macos "x86_64-apple-darwin" "x86_64" "10.12")
build-macos-arm64: (_build-macos "aarch64-apple-darwin" "arm64" "11.0")

# requires node + appdmg (npm i -g appdmg)
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
