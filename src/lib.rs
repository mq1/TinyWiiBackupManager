// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

pub const PRODUCT_NAME: &str = "TinyWiiBackupManager";

mod app;
mod components;
pub mod error_handling;
mod game;
mod titles;

pub use app::App;
