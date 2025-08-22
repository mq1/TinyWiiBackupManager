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
            "linuxdeploy-x86_64.AppImage",
            "--appdir",
            f"{PRODUCT_NAME}.AppDir",
            "--executable",
            Path("target") / f"{arch}-unknown-linux-gnu" / "release" / NAME,
            "--desktop-file",
            f"{PRODUCT_NAME}.desktop",
            "--icon-file",
            Path("assets") / "linux" / "32x32" / f"{NAME}.png",
            "--icon-file",
            Path("assets") / "linux" / "48x48" / f"{NAME}.png",
            "--icon-file",
            Path("assets") / "linux" / "64x64" / f"{NAME}.png",
            "--icon-file",
            Path("assets") / "linux" / "128x128" / f"{NAME}.png",
            "--icon-file",
            Path("assets") / "linux" / "256x256" / f"{NAME}.png",
            "--icon-file",
            Path("assets") / "linux" / "512x512" / f"{NAME}.png",
            "--output",
            "appimage",
        ],
        env={"ARCH": arch},
        check=True,
    )

    print("Renaming AppImage...")
    Path(f"{PRODUCT_NAME}-{arch}.AppImage").rename(
        Path("dist") / f"{SHORT_NAME}-{VERSION}-Linux-{arch}.AppImage"
    )

    print("Cleaning up...")
    rmtree(f"{PRODUCT_NAME}.AppDir", ignore_errors=True)
    Path(f"{PRODUCT_NAME}.desktop").unlink(missing_ok=True)

    print("Creating tarball...")
    with tarfile.open(
        Path("dist") / f"{SHORT_NAME}-{VERSION}-Linux-{arch}.tar.gz", "w:gz"
    ) as tar:
        tar.add(
            Path("target") / f"{arch}-unknown-linux-gnu" / "release" / NAME,
            arcname=f"{SHORT_NAME}-{VERSION}-Linux-{arch}",
        )

    print(f"✅ Created {SHORT_NAME}-{VERSION}-Linux-{arch}.tar.gz in ./dist")


def linux():
    # Build for both architectures
    for arch in ["x86_64", "aarch64"]:
        _linux(arch)


def windows():
    # Build for both architectures
    for arch in ["x86_64", "aarch64"]:
        _windows(arch)


def _windows(arch: str):
    import zipfile

    print(f"Building for {arch}...")
    run(
        ["cargo", "build", "--release", "--target", f"{arch}-pc-windows-msvc"],
        check=True,
    )

    print("Creating .zip...")
    with zipfile.ZipFile(
        Path("dist") / f"{SHORT_NAME}-{VERSION}-Windows-{arch}.zip",
        "w",
        zipfile.ZIP_DEFLATED,
    ) as zip:
        zip.write(
            Path("target") / f"{arch}-pc-windows-msvc" / "release" / f"{NAME}.exe",
            arcname=f"{SHORT_NAME}-{VERSION}-Windows-{arch}.exe",
        )

    print(f"✅ Created {SHORT_NAME}-{VERSION}-Windows-{arch}.zip in ./dist")


def macos():
    import plistlib

    print("(Re-)creating .app bundle...")
    rmtree(f"{PRODUCT_NAME}.app", ignore_errors=True)
    (Path(f"{PRODUCT_NAME}.app") / "Contents" / "MacOS").mkdir(parents=True)
    (Path(f"{PRODUCT_NAME}.app") / "Contents" / "Resources").mkdir(parents=True)

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
            Path(f"{PRODUCT_NAME}.app") / "Contents" / "MacOS" / NAME,
            Path("target") / "aarch64-apple-darwin" / "release" / NAME,
            Path("target") / "x86_64-apple-darwin" / "release" / NAME,
        ],
        check=True,
    )

    print("Creating Info.plist...")
    with open(Path(f"{PRODUCT_NAME}.app") / "Contents" / "Info.plist", "wb") as f:
        plistlib.dump(
            {
                "CFBundleName": PRODUCT_NAME,
                "CFBundleDisplayName": PRODUCT_NAME,
                "CFBundleIdentifier": BUNDLE_ID,
                "CFBundleVersion": VERSION,
                "CFBundleShortVersionString": VERSION,
                "CFBundlePackageType": "APPL",
                "CFBundleExecutable": NAME,
                "CFBundleIconFile": NAME,
                "LSMinimumSystemVersion": "11",
                "NSHumanReadableCopyright": LEGAL_COPYRIGHT,
                "NSPrincipalClass": "NSApplication",
                "NSHighResolutionCapable": True,
                "NSSupportsAutomaticGraphicsSwitching": True,
            },
            f,
        )

    print("Copying assets...")
    copy(
        Path("assets") / "macos" / f"{NAME}.icns",
        Path(f"{PRODUCT_NAME}.app") / "Contents" / "Resources",
    )

    print("Creating .dmg...")
    run(
        [
            "npx",
            "--yes",
            "create-dmg@7.0.0",
            f"{PRODUCT_NAME}.app",
            ".",
            "--overwrite",
        ]
    )

    print("Renaming .dmg...")
    Path(f"{PRODUCT_NAME} {VERSION}.dmg").rename(
        Path("dist") / f"{SHORT_NAME}-{VERSION}-MacOS-Universal2.dmg"
    )

    print("Cleaning up...")
    rmtree(f"{PRODUCT_NAME}.app")

    print(f"✅ Created {SHORT_NAME}-{VERSION}-MacOS-Universal2.dmg in ./dist")


Path("dist").mkdir(exist_ok=True)
globals()[sys.argv[1]]()
