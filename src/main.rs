// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod archive;
mod checksum;
mod config;
mod convert;
mod covers;
mod data_dir;
mod dialogs;
mod disc_info;
mod drive_info;
mod extensions;
mod game;
mod homebrew_app;
mod homebrew_app_meta;
mod id_map;
mod logic;
mod notification;
mod osc;
mod scrub;
mod standard_conversion;
mod util;

#[cfg(windows)]
mod window_color;

#[cfg(windows)]
mod xp_dialogs;

use crate::data_dir::DATA_DIR;
use anyhow::{Result, bail};
use slint::ComponentHandle;
use std::{process::Command, sync::LazyLock};
use ureq::{
    Agent,
    tls::{RootCerts, TlsConfig, TlsProvider},
};

slint::include_modules!();

const UREQ_AGENT: LazyLock<Agent> = LazyLock::new(|| {
    const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

    #[cfg(feature = "native-tls")]
    let provider = TlsProvider::NativeTls;

    #[cfg(feature = "rustls")]
    let provider = TlsProvider::Rustls;

    #[cfg(feature = "native-tls")]
    let certs = RootCerts::PlatformVerifier;

    #[cfg(feature = "rustls")]
    let certs = RootCerts::WebPki;

    Agent::config_builder()
        .user_agent(USER_AGENT)
        .tls_config(
            TlsConfig::builder()
                .provider(provider)
                .root_certs(certs)
                .build(),
        )
        .build()
        .new_agent()
});

fn restart_with_sw_rendering() -> Result<()> {
    let exe = std::env::current_exe()?;

    let mut cmd = Command::new(exe);
    cmd.env("SLINT_BACKEND", "winit-software");

    let _ = cmd.spawn()?;

    std::process::exit(0);
}

fn main() -> Result<()> {
    if DATA_DIR.as_os_str().is_empty() {
        bail!("Failed to get data dir");
    }

    let config = Config::load();
    let app = AppWindow::new()?;
    app.global::<Logic<'_>>().init(config, app.window());

    if let Err(e) = app.run() {
        if std::env::var("SLINT_BACKEND").unwrap_or_default() == "winit-software" {
            bail!(e);
        }

        return restart_with_sw_rendering();
    }

    Ok(())
}
