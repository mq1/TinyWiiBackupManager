#!/usr/bin/env python
import sys
import tomllib
from subprocess import run
from shutil import rmtree, copy
from pathlib import Path


# Load configuration
with open("Cargo.toml", "rb") as f:
    config = tomllib.load(f)


# Shared constants
NAME = config["package"]["name"]
VERSION = config["package"]["version"]
DESCRIPTION = config["package"]["description"]
SHORT_NAME = config["package"]["metadata"]["ShortName"]
BUNDLE_ID = config["package"]["metadata"]["BundleIdentifier"]
PRODUCT_NAME = config["package"]["metadata"]["winres"]["ProductName"]
LEGAL_COPYRIGHT = config["package"]["metadata"]["winres"]["LegalCopyright"]


def _linux(arch: str):
    import tarfile

    print(f"Building for {arch}...")
    run(
        ["cargo", "build", "--release", "--target", f"{arch}-unknown-linux-gnu"],
        check=True,
    )

    print("Creating desktop file...")
    with open(f"{PRODUCT_NAME}.desktop", "w") as f:
        f.write(f"""[Desktop Entry]
Name={PRODUCT_NAME}
Icon={NAME}
Categories=Utility
Comment={DESCRIPTION}
Type=Application
Exec={NAME}
""")

    print("Creating AppImage...")
    run(
        [
            f"linuxdeploy-{arch}.AppImage",
            "--appdir",
            f"{PRODUCT_NAME}.AppDir",
            "--executable",
            f"target/{arch}-unknown-linux-gnu/release/{NAME}",
            "--desktop-file",
            f"{PRODUCT_NAME}.desktop",
            "--icon-file",
            f"assets/linux/32x32/{NAME}.png",
            "--icon-file",
            f"assets/linux/48x48/{NAME}.png",
            "--icon-file",
            f"assets/linux/64x64/{NAME}.png",
            "--icon-file",
            f"assets/linux/128x128/{NAME}.png",
            "--icon-file",
            f"assets/linux/256x256/{NAME}.png",
            "--icon-file",
            f"assets/linux/512x512/{NAME}.png",
            "--output",
            "appimage",
        ],
        check=True,
    )

    print("Renaming AppImage...")
    Path(f"{PRODUCT_NAME}-{arch}.AppImage").rename(
        f"dist/{SHORT_NAME}-{VERSION}-Linux-{arch}.AppImage"
    )

    print("Cleaning up...")
    rmtree(f"{PRODUCT_NAME}.AppDir", ignore_errors=True)
    Path(f"{PRODUCT_NAME}.desktop").unlink(missing_ok=True)

    print("Creating tarball...")
    with tarfile.open(
        f"dist/{SHORT_NAME}-{VERSION}-Linux-{arch}.tar.gz", "w:gz"
    ) as tar:
        tar.add(
            f"target/{arch}-unknown-linux-gnu/release/{NAME}",
            arcname=f"{SHORT_NAME}-{VERSION}-Linux-{arch}",
        )

    print(f"✅ Created {SHORT_NAME}-{VERSION}-Linux-{arch}.tar.gz in ./dist")


def linux_aarch64():
    _linux("aarch64")


def linux_x86_64():
    _linux("x86_64")


def _windows(arch: str):
    import zipfile

    print(f"Building for {arch}...")
    run(
        ["cargo", "build", "--release", "--target", f"{arch}-pc-windows-msvc"],
        check=True,
    )

    print("Creating .zip...")
    with zipfile.ZipFile(
        f"dist/{SHORT_NAME}-{VERSION}-Windows-{arch}.zip", "w", zipfile.ZIP_DEFLATED
    ) as zip:
        zip.write(
            f"target/{arch}-pc-windows-msvc/release/{NAME}.exe",
            arcname=f"{SHORT_NAME}-{VERSION}-Windows-{arch}.exe",
        )

    print(f"✅ Created {SHORT_NAME}-{VERSION}-Windows-{arch}.zip in ./dist")


def windows_x86_64():
    _windows("x86_64")


def windows_aarch64():
    _windows("aarch64")


def macos_universal2():
    import plistlib

    print("(Re-)creating .app bundle...")
    rmtree(f"{PRODUCT_NAME}.app", ignore_errors=True)
    Path(f"{PRODUCT_NAME}.app/Contents/MacOS").mkdir(parents=True)
    Path(f"{PRODUCT_NAME}.app/Contents/Resources").mkdir(parents=True)

    print("Building for aarch64...")
    run(["cargo", "build", "--release", "--target", "aarch64-apple-darwin"], check=True)

    print("Building for x86_64...")
    run(["cargo", "build", "--release", "--target", "x86_64-apple-darwin"], check=True)

    print("Creating universal binary...")
    run(
        [
            "lipo",
            "-create",
            "-output",
            f"{PRODUCT_NAME}.app/Contents/MacOS/{NAME}",
            f"target/aarch64-apple-darwin/release/{NAME}",
            f"target/x86_64-apple-darwin/release/{NAME}",
        ],
        check=True,
    )

    print("Creating Info.plist...")

    info = {
        "CFBundleName": PRODUCT_NAME,
        "CFBundleDisplayName": PRODUCT_NAME,
        "CFBundleIdentifier": BUNDLE_ID,
        "CFBundleVersion": VERSION,
        "CFBundleShortVersionString": VERSION,
        "CFBundlePackageType": "APPL",
        "CFBundleExecutable": NAME,
        "CFBundleIconFile": NAME,
        "LSMinimumSystemVersion": "11.0",
        "NSHumanReadableCopyright": LEGAL_COPYRIGHT,
        "NSPrincipalClass": "NSApplication",
        "NSHighResolutionCapable": True,
        "NSSupportsAutomaticGraphicsSwitching": True,
    }

    plistlib.dump(info, open(f"{PRODUCT_NAME}.app/Contents/Info.plist", "wb"))

    print("Copying assets...")
    copy(
        f"assets/macos/{NAME}.icns",
        f"{PRODUCT_NAME}.app/Contents/Resources/{NAME}.icns",
    )

    print("Creating .dmg...")
    run(
        [
            "npx",
            "--yes",
            "create-dmg",
            f"{PRODUCT_NAME}.app",
            ".",
            "--overwrite",
        ]
    )

    print("Renaming .dmg...")
    Path(f"{PRODUCT_NAME} {VERSION}.dmg").rename(
        f"dist/{SHORT_NAME}-{VERSION}-MacOS-Universal2.dmg"
    )

    print("Cleaning up...")
    rmtree(f"dist/{PRODUCT_NAME}.app")

    print(f"✅ Created {SHORT_NAME}-{VERSION}-MacOS-Universal2.dmg in ./dist")


Path("dist").mkdir(exist_ok=True)
globals()[sys.argv[1]]()
