#!/usr/bin/env python3
import sys
import tomllib
from subprocess import run
from shutil import copy
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


def linux():
    import tarfile
    import tempfile

    # Package each architecture
    for arch in ["x86_64", "aarch64"]:
        # Create a temporary directory for this build
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_path = Path(temp_dir)
            appdir_path = temp_path / f"{PRODUCT_NAME}.AppDir"
            desktop_path = temp_path / f"{PRODUCT_NAME}.desktop"

            print(f"Packaging for {arch}...")

            print("Creating desktop file...")
            with open(desktop_path, "w") as f:
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
                    appdir_path,
                    "--executable",
                    (
                        Path("target") / f"{arch}-unknown-linux-gnu" / "release" / NAME
                    ).resolve(),
                    "--desktop-file",
                    desktop_path,
                    "--icon-file",
                    (Path("assets") / "linux" / "32x32" / f"{NAME}.png").resolve(),
                    "--icon-file",
                    (Path("assets") / "linux" / "48x48" / f"{NAME}.png").resolve(),
                    "--icon-file",
                    (Path("assets") / "linux" / "64x64" / f"{NAME}.png").resolve(),
                    "--icon-file",
                    (Path("assets") / "linux" / "128x128" / f"{NAME}.png").resolve(),
                    "--icon-file",
                    (Path("assets") / "linux" / "256x256" / f"{NAME}.png").resolve(),
                    "--icon-file",
                    (Path("assets") / "linux" / "512x512" / f"{NAME}.png").resolve(),
                    "--output",
                    "appimage",
                ],
                env={"ARCH": arch},
                check=True,
                cwd=temp_dir,  # Run in the temporary directory
            )

            print("Renaming AppImage...")
            (temp_path / f"{PRODUCT_NAME}-{arch}.AppImage").rename(
                Path("dist") / f"{SHORT_NAME}-{VERSION}-Linux-{arch}.AppImage"
            )

            print("Creating tarball...")
            with tarfile.open(
                Path("dist") / f"{SHORT_NAME}-{VERSION}-Linux-{arch}.tar.gz", "w:gz"
            ) as tar:
                tar.add(
                    Path("target") / f"{arch}-unknown-linux-gnu" / "release" / NAME,
                    arcname=f"{SHORT_NAME}-{VERSION}-Linux-{arch}",
                )

            print(f"✅ Created {SHORT_NAME}-{VERSION}-Linux-{arch}.tar.gz in ./dist")


def windows():
    import zipfile

    # Package each architecture
    for arch in ["x86_64", "aarch64"]:
        print(f"Packaging for {arch}...")

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
    import tempfile

    # Create universal binary and app bundle
    with tempfile.TemporaryDirectory() as temp_dir:
        temp_path = Path(temp_dir)
        app_bundle_path = temp_path / f"{PRODUCT_NAME}.app"

        print("Creating .app bundle...")
        (app_bundle_path / "Contents" / "MacOS").mkdir(parents=True)
        (app_bundle_path / "Contents" / "Resources").mkdir(parents=True)

        print("Creating universal binary...")
        run(
            [
                "lipo",
                "-create",
                "-output",
                app_bundle_path / "Contents" / "MacOS" / NAME,
                Path("target") / "aarch64-apple-darwin" / "release" / NAME,
                Path("target") / "x86_64-apple-darwin" / "release" / NAME,
            ],
            check=True,
        )

        print("Creating Info.plist...")
        with open(app_bundle_path / "Contents" / "Info.plist", "wb") as f:
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
            app_bundle_path / "Contents" / "Resources",
        )

        print("Creating .dmg...")
        run(
            [
                "npx",
                "--yes",
                "create-dmg@7.0.0",
                app_bundle_path,
                temp_path,
                "--overwrite",
            ]
        )

        print("Renaming .dmg...")
        (temp_path / f"{PRODUCT_NAME} {VERSION}.dmg").rename(
            Path("dist") / f"{SHORT_NAME}-{VERSION}-MacOS-Universal2.dmg"
        )

        print(f"✅ Created {SHORT_NAME}-{VERSION}-MacOS-Universal2.dmg in ./dist")


Path("dist").mkdir(exist_ok=True)
globals()[sys.argv[1]]()
