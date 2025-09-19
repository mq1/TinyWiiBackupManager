// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use std::convert::Into;
use std::sync::LazyLock;
use std::time::Duration;
use const_format::concatcp;
use ureq::Agent;

pub mod app;
mod base_dir;
pub mod game;
mod gui;
mod messages;
mod settings;
mod task;
mod util;


pub const PRODUCT_NAME: &str = "TinyWiiBackupManager";
pub static AGENT: LazyLock<Agent> = LazyLock::new(||
    Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(10)))
        .user_agent(concatcp!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")))
        .build()
        .into()
);
