// SPDX-FileCopyrightText: 2023 Manuel Quarneti <manuel.quarneti@proton.me>
// SPDX-License-Identifier: GPL-2.0-only

fn main() {
    if cfg!(target_os = "macos") {
        cc::Build::new()
            .file("wbfs_file_2.9_64bit/wbfs.c")
            .file("wbfs_file_2.9_64bit/tools.c")
            .file("wbfs_file_2.9_64bit/bn.c")
            .file("wbfs_file_2.9_64bit/ec.c")
            .file("wbfs_file_2.9_64bit/libwbfs/libwbfs.c")
            .file("wbfs_file_2.9_64bit/libwbfs/wiidisc.c")
            .file("wbfs_file_2.9_64bit/libwbfs/rijndael.c")
            .file("wbfs_file_2.9_64bit/splits.c")
            .file("wbfs_file_2.9_64bit/libwbfs/libwbfs_osx.c")
            .include("/opt/homebrew/opt/openssl@3/include")
            .include("wbfs_file_2.9_64bit")
            .include("wbfs_file_2.9_64bit/libwbfs")
            .define("LARGE_FILES", None)
            .define("_FILE_OFFSET_BITS", "64")
            .opt_level(2)
            .compile("wbfs_file")
    }

    if cfg!(target_os = "linux") {
        cc::Build::new()
            .file("wbfs_file_2.9_64bit/wbfs.c")
            .file("wbfs_file_2.9_64bit/tools.c")
            .file("wbfs_file_2.9_64bit/bn.c")
            .file("wbfs_file_2.9_64bit/ec.c")
            .file("wbfs_file_2.9_64bit/libwbfs/libwbfs.c")
            .file("wbfs_file_2.9_64bit/libwbfs/wiidisc.c")
            .file("wbfs_file_2.9_64bit/libwbfs/rijndael.c")
            .file("wbfs_file_2.9_64bit/splits.c")
            .file("wbfs_file_2.9_64bit/libwbfs/libwbfs_linux.c")
            .include("/usr/include/openssl")
            .include("wbfs_file_2.9_64bit")
            .include("wbfs_file_2.9_64bit/libwbfs")
            .define("LARGE_FILES", None)
            .define("_FILE_OFFSET_BITS", "64")
            .opt_level(2)
            .compile("wbfs_file")
    }

    if cfg!(target_os = "windows") {
        cc::Build::new()
            .file("wbfs_file_2.9_64bit/wbfs.c")
            .file("wbfs_file_2.9_64bit/tools.c")
            .file("wbfs_file_2.9_64bit/bn.c")
            .file("wbfs_file_2.9_64bit/ec.c")
            .file("wbfs_file_2.9_64bit/libwbfs/libwbfs.c")
            .file("wbfs_file_2.9_64bit/libwbfs/wiidisc.c")
            .file("wbfs_file_2.9_64bit/libwbfs/rijndael.c")
            .file("wbfs_file_2.9_64bit/splits.c")
            .file("wbfs_file_2.9_64bit/libwbfs/libwbfs_win32.c")
            .include("wbfs_file_2.9_64bit")
            .include("wbfs_file_2.9_64bit/libwbfs")
            .define("LARGE_FILES", None)
            .define("_FILE_OFFSET_BITS", "64")
            .opt_level(2)
            .compile("wbfs_file")
    }
}
