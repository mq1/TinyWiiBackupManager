// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::Path;

#[rustfmt::skip]
pub const SUPPORTED_INPUT_EXTENSIONS: &[&str] = &[
    "gcm", "iso", "wbfs", "wia", "rvz", "ciso", "gcz", "tgc", "nfs",
    "GCM", "ISO", "WBFS", "WIA", "RVZ", "CISO", "GCS", "TGC", "NFS",
];

pub fn get_filter() -> Box<fn(&Path) -> bool> {
    Box::new(|path: &Path| {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| SUPPORTED_INPUT_EXTENSIONS.contains(&ext))
            .unwrap_or(false)
    })
}
