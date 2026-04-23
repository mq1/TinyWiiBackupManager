// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use nod::common::{Compression, Format};
use nod::write::FormatOptions;
use std::ffi::OsStr;

pub const INPUT_DIALOG_FILTER: (&str, &[&str]) = (
    "Nintendo Optical Disc",
    &[
        "gcm", "iso", "wbfs", "wia", "rvz", "ciso", "gcz", "tgc", "nfs", "zip",
    ],
);

pub const OUTPUT_DIALOG_FILTER: (&str, &[&str]) = (
    "Nintendo Optical Disc",
    &[
        "gcm", "iso", "wbfs", "wia", "rvz", "ciso", "gcz", "tgc", "nfs",
    ],
);

pub fn str_to_format(s: &str) -> Option<Format> {
    match s.to_ascii_lowercase().as_str() {
        "iso" | "gcm" => Some(Format::Iso),
        "ciso" => Some(Format::Ciso),
        "gcz" => Some(Format::Gcz),
        "nfs" => Some(Format::Nfs),
        "rvz" => Some(Format::Rvz),
        "wbfs" => Some(Format::Wbfs),
        "wia" => Some(Format::Wia),
        "tgc" => Some(Format::Tgc),
        _ => None,
    }
}

pub fn ext_to_format(ext: &OsStr) -> Option<Format> {
    ext.to_str().and_then(str_to_format)
}

pub fn format_to_opts(format: Format) -> FormatOptions {
    match format {
        Format::Rvz => FormatOptions {
            format: Format::Rvz,
            compression: Compression::Zstandard(19),
            block_size: Format::Rvz.default_block_size(),
        },
        format => FormatOptions::new(format),
    }
}
