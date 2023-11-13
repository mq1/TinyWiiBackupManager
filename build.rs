// SPDX-FileCopyrightText: 2023 Manuel Quarneti <manuel.quarneti@proton.me>
// SPDX-License-Identifier: GPL-2.0-only

fn main() {
    cc::Build::new()
        .file("wbfs_file_2.9/wbfs.c")
        .file("wbfs_file_2.9/tools.c")
        .file("wbfs_file_2.9/bn.c")
        .file("wbfs_file_2.9/ec.c")
        .file("wbfs_file_2.9/libwbfs/libwbfs.c")
        .file("wbfs_file_2.9/libwbfs/wiidisc.c")
        .file("wbfs_file_2.9/libwbfs/rijndael.c")
        .file("wbfs_file_2.9/splits.c")
        .file("wbfs_file_2.9/libwbfs/libwbfs_linux.c")
        .file("wbfs_file_2.9/libwbfs/libwbfs_osx.c")
        .file("wbfs_file_2.9/libwbfs/libwbfs_win32.c")
        .include("wbfs_file_2.9")
        .include("wbfs_file_2.9/libwbfs")
        .define("LARGE_FILES", None)
        .define("_FILE_OFFSET_BITS", "64")
        .opt_level_str("z")
        .flag("-flto=thin")
        .compile("wbfs_file")
}
