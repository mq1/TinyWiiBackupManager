// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::Path;

#[rustfmt::skip]
pub const SUPPORTED_INPUT_EXTENSIONS: &[&str] = &[
    "gcm", "iso", "wbfs", "wia", "rvz", "ciso", "gcz", "tgc", "nfs",
    "GCM", "ISO", "WBFS", "WIA", "RVZ", "CISO", "GCS", "TGC", "NFS",
];

pub fn game_extension_filter(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| SUPPORTED_INPUT_EXTENSIONS.contains(&ext))
        .unwrap_or(false)
}

pub fn zip_extension_filter(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext == "zip")
        .unwrap_or(false)
}

pub fn hbc_app_extension_filter(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext == "zip" || ext == "dol" || ext == "elf")
        .unwrap_or(false)
}
