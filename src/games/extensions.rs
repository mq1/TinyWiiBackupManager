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

pub fn ext_to_format(ext: &OsStr) -> Option<Format> {
    if ext.eq_ignore_ascii_case("gcm") || ext.eq_ignore_ascii_case("iso") {
        Some(Format::Iso)
    } else if ext.eq_ignore_ascii_case("wbfs") {
        Some(Format::Wbfs)
    } else if ext.eq_ignore_ascii_case("wia") {
        Some(Format::Wia)
    } else if ext.eq_ignore_ascii_case("rvz") {
        Some(Format::Rvz)
    } else if ext.eq_ignore_ascii_case("ciso") {
        Some(Format::Ciso)
    } else if ext.eq_ignore_ascii_case("gcz") {
        Some(Format::Gcz)
    } else if ext.eq_ignore_ascii_case("tgc") {
        Some(Format::Tgc)
    } else if ext.eq_ignore_ascii_case("nfs") {
        Some(Format::Nfs)
    } else {
        None
    }
}

pub fn format_to_ext(format: Format) -> &'static str {
    match format {
        Format::Iso => "iso",
        Format::Wbfs => "wbfs",
        Format::Wia => "wia",
        Format::Rvz => "rvz",
        Format::Ciso => "ciso",
        Format::Gcz => "gcz",
        Format::Tgc => "tgc",
        Format::Nfs => "nfs",
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
