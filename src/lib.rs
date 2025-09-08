// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use const_format::concatcp;

pub const PRODUCT_NAME: &str = "TinyWiiBackupManager";
pub const USER_AGENT: &str = concatcp!(PRODUCT_NAME, "/", env!("CARGO_PKG_VERSION"));

pub mod app;
mod base_dir;
mod game;
mod gui;
mod messages;
mod settings;
mod task;
mod util;
