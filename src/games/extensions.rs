// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use nod::common::{Compression, Format};
use nod::write::FormatOptions;
use std::ffi::OsStr;

pub const SUPPORTED_INPUT_EXTENSIONS: &[&str] = &[
    "gcm", "iso", "wbfs", "wia", "rvz", "ciso", "gcz", "tgc", "nfs", "zip",
];

pub const SUPPORTED_DISC_EXTENSIONS: &[&str] = &[
    "gcm", "iso", "wbfs", "wia", "rvz", "ciso", "gcz", "tgc", "nfs",
];

pub fn ext_to_format(ext: Option<&OsStr>) -> Option<Format> {
    match ext.and_then(OsStr::to_str) {
        Some("gcm" | "iso") => Some(Format::Iso),
        Some("wbfs") => Some(Format::Wbfs),
        Some("wia") => Some(Format::Wia),
        Some("rvz") => Some(Format::Rvz),
        Some("ciso") => Some(Format::Ciso),
        Some("gcz") => Some(Format::Gcz),
        Some("tgc") => Some(Format::Tgc),
        Some("nfs") => Some(Format::Nfs),
        _ => None,
    }
}

pub fn format_to_opts(format: Format) -> FormatOptions {
    match format {
        Format::Iso => FormatOptions::new(Format::Iso),
        Format::Wbfs => FormatOptions::new(Format::Wbfs),
        Format::Wia => FormatOptions::new(Format::Wia),
        Format::Rvz => FormatOptions {
            format: Format::Rvz,
            compression: Compression::Zstandard(19),
            block_size: Format::Rvz.default_block_size(),
        },
        Format::Ciso => FormatOptions::new(Format::Ciso),
        Format::Gcz => FormatOptions::new(Format::Gcz),
        Format::Tgc => FormatOptions::new(Format::Tgc),
        Format::Nfs => FormatOptions::new(Format::Nfs),
    }
}
