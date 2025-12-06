// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

#[rustfmt::skip]
pub const SUPPORTED_INPUT_EXTENSIONS: &[&str] = &[
    "gcm", "iso", "wbfs", "wia", "rvz", "ciso", "gcz", "tgc", "nfs", "zip",
    "GCM", "ISO", "WBFS", "WIA", "RVZ", "CISO", "GCS", "TGC", "NFS", "ZIP",
];

#[rustfmt::skip]
pub const ZIP_EXTENSIONS: &[&str] = &[
    "zip",
    "ZIP",
];

#[rustfmt::skip]
pub const HBC_APP_EXTENSIONS: &[&str] = &[
    "zip", "dol", "elf",
    "ZIP", "DOL", "ELF",
];
