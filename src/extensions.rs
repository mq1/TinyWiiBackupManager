// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{ArchiveFormat, WiiOutputFormat, config};

pub const SUPPORTED_INPUT_EXTENSIONS: &[&str] = &[
    "gcm", "iso", "wbfs", "wia", "rvz", "ciso", "gcz", "tgc", "nfs",
];

impl ArchiveFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            ArchiveFormat::Rvz => "rvz",
            ArchiveFormat::Iso => "iso",
        }
    }
}

impl WiiOutputFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            WiiOutputFormat::WbfsAuto | WiiOutputFormat::WbfsFixed => "wbfs",
            WiiOutputFormat::Iso => "iso",
        }
    }
}

pub fn get_convert_extension(is_wii: bool) -> &'static str {
    if is_wii {
        config::get().wii_output_format.extension()
    } else {
        "iso"
    }
}
