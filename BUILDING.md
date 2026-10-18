# 🛠️ Building TinyWiiBackupManager

## 🦀 Standard build

**Prerequisites**
- [Rust](https://rust-lang.org/)
- A C compiler

**Steps**
```sh
git clone https://github.com/mq1/TinyWiiBackupManager.git
cd TinyWiiBackupManager
cargo build --release
```

The binary will be at `./target/release/TinyWiiBackupManager`

## 📦 Distribution build

This will be built with linker-plugin-lto for more efficient optimization. \
Windows builds will be compatible with Windows XP. \
Linux builds can be built on manylinux2014 or newer.

**Prerequisites**
- [mise-en-place](https://mise.jdx.dev/)

**Steps**
```sh
git clone https://github.com/mq1/TinyWiiBackupManager.git
cd TinyWiiBackupManager
mise trust
mise install
mise run build-<os>-<arch>
mise run package-<os>-<arch>
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

