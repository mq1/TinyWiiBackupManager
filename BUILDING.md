# 🛠️ Building Ultimate Game Backup Manager

## 🦀 Standard build

**Prerequisites**
- [Rust](https://rust-lang.org/)
- A C compiler

**Steps**
```sh
git clone https://github.com/petter1/UltimateGameBackupManager.git
cd UltimateGameBackupManager
cargo build --release
```

The binary will be at `./target/release/UltimateGameBackupManager`

## 📦 Distribution build

This will be built with linker-plugin-lto for more in-depth optimization. \
Windows builds will be compatible with Windows XP. \
Linux builds can be built on manylinux2014 or newer.

**Prerequisites**
- [just](https://just.systems/)
- [Rust 1.99](https://rust-lang.org/) with the `rust-src` component
- [LLVM 23](https://llvm.org/)
- [NASM](https://www.nasm.us/) (for build-windows-x86)
- [appimagetool](https://github.com/AppImage/appimagetool) (for package-linux-*)
- [node-appdmg](https://github.com/LinusU/node-appdmg) (for package-macos-universal)

For linux builds, dockerfiles with all required tools are provided in `./dockerfiles`).

**Steps**
```sh
git clone https://github.com/petter1/UltimateGameBackupManager.git
cd UltimateGameBackupManager
just build-<os>-<arch>
just package-<os>-<arch>
```

The available os-arch pairs are:
- windows-x64
- windows-arm64
- windows-x86
- linux-x86_64
- linux-aarch64
- linux-i686
- macos-x86_64 (only build)
- macos-arm64 (only build)
- macos-universal (only package)

The binary will be in `./out` \
The distributable artifact will be in `./dist`

