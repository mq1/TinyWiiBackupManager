// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{message::Message, state::State};
use anyhow::{Result, bail};
use iced::{Task, futures::TryFutureExt};
use std::{
    ffi::OsStr,
    fs,
    net::Ipv4Addr,
    path::{Path, PathBuf},
    sync::Arc,
};

fn send_too_wiiload(wii_ip: &str, path: &Path) -> Result<String> {
    let wii_ip: Ipv4Addr = wii_ip.parse()?;

    let Some(filename) = path.file_name().and_then(OsStr::to_str) else {
        bail!("Failed to get filename")
    };

    let Some(ext) = path.extension().and_then(OsStr::to_str) else {
        bail!("Failed to get extension")
    };

    let body = fs::read(path)?;

    if ext == "zip" {
        wiiload::send(filename, body, wii_ip)?;
    } else {
        wiiload::compress_then_send(filename, body, wii_ip)?;
    }

    Ok("File sent successfully".to_string())
}

pub fn get_send_to_wiiload_task(state: &State, path: PathBuf) -> Task<Message> {
    let wii_ip = state.config.wii_ip().clone();

    Task::perform(
        async move { send_too_wiiload(&wii_ip, &path) }.map_err(Arc::new),
        Message::GenericResult,
    )
}
