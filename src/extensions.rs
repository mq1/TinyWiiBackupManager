// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::ArchiveFormat;

#[rustfmt::skip]
pub const SUPPORTED_INPUT_EXTENSIONS: &[&str] = &[
    "gcm", "iso", "wbfs", "wia", "rvz", "ciso", "gcz", "tgc", "nfs",
    "GCM", "ISO", "WBFS", "WIA", "RVZ", "CISO", "GCS", "TGC", "NFS",
];

impl ArchiveFormat {
    pub fn extension(self) -> &'static str {
        match self {
            ArchiveFormat::Rvz => "rvz",
            ArchiveFormat::Iso => "iso",
        }
    }
}
