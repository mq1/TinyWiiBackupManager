// SPDX-FileCopyrightText: 2023 Manuel Quarneti <manuel.quarneti@proton.me>
// SPDX-License-Identifier: GPL-2.0-only

use std::ffi::CString;

extern "C" {
    fn conv_to_wbfs(
        filename: *const ::std::os::raw::c_char,
        dest_dir: *const ::std::os::raw::c_char,
    );
}

pub fn conv_to_wbfs_wrapper(src: &str, dest: &str) {
    let src = CString::new(src).unwrap();
    let dest = CString::new(dest).unwrap();

    unsafe {
        conv_to_wbfs(src.as_ptr(), dest.as_ptr());
    };
}
