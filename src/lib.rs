// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

pub const PRODUCT_NAME: &str = "TinyWiiBackupManager";

pub mod app;
mod base_dir;
mod components;
mod convert;
mod cover_manager;
mod game;
mod jobs;
mod messages;
mod titles;
mod update_check;
mod util;

#[rustfmt::skip]
pub const SUPPORTED_INPUT_EXTENSIONS: &[&str] = &[
    "gcm", "GCM",
    "iso", "ISO",
    "wbfs", "WBFS",
    "wia", "WIA",
    "rvz", "RVZ",
    "ciso", "CISO",
    "gcz", "GCZ",
    "tgc", "TGC",
    "nfs", "NFS",
];
