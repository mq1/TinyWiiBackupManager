// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use nod::common::{Compression, Format};
use nod::write::FormatOptions;

pub const SUPPORTED_INPUT_EXTENSIONS: &[&str] = &[
    "gcm", "iso", "wbfs", "wia", "rvz", "ciso", "gcz", "tgc", "nfs", "zip",
];

pub const SUPPORTED_DISC_EXTENSIONS: &[&str] = &[
    "gcm", "iso", "wbfs", "wia", "rvz", "ciso", "gcz", "tgc", "nfs",
];

pub const ZIP_EXTENSIONS: &[&str] = &["zip"];

pub const HBC_APP_EXTENSIONS: &[&str] = &["zip", "dol", "elf"];

pub fn ext_to_format(ext: &str) -> Option<Format> {
    match ext {
        "gcm" | "iso" => Some(Format::Iso),
        "wbfs" => Some(Format::Wbfs),
        "wia" => Some(Format::Wia),
        "rvz" => Some(Format::Rvz),
        "ciso" => Some(Format::Ciso),
        "gcz" => Some(Format::Gcz),
        "tgc" => Some(Format::Tgc),
        "nfs" => Some(Format::Nfs),
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
