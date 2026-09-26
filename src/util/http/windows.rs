// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::util::http::USER_AGENT;
use anyhow::{Result, bail};
use winhttp::{Session, SessionConfig, WinResult, crack_url};

fn session() -> WinResult<Session> {
    Session::with_config(SessionConfig {
        user_agent: USER_AGENT.to_string(),
        ..Default::default()
    })
}

pub fn download(url: &str, mut dest: impl std::io::Write) -> Result<()> {
    let url_components = crack_url(url)?;

    let session = session()?;
    let connection = session.connect(&url_components.host, url_components.port)?;

    let request = connection
        .request("GET", &url_components.path)
        .secure()
        .build()?;

    request.send()?;
    request.receive_response()?;

    let status = request.status_code()?;
    if status >= 400 {
        bail!("HTTP error: {status}");
    }

    let mut buf = [0u8; 64 * 1024];
    loop {
        let read = request.read_data(&mut buf)?;
        if read == 0 {
            break; // EOF
        }
        dest.write_all(&buf[..read])?;
    }

    Ok(())
}

pub fn post_then_download(url: &str, body: &[u8], mut dest: impl std::io::Write) -> Result<()> {
    let url_components = crack_url(url)?;

    let session = session()?;
    let connection = session.connect(&url_components.host, url_components.port)?;

    let request = connection
        .request("POST", &url_components.path)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .secure()
        .build()?;

    request.send_with_body(body)?;
    request.receive_response()?;

    let status = request.status_code()?;
    if status >= 400 {
        bail!("HTTP error: {status}");
    }

    let mut buf = [0u8; 64 * 1024];
    loop {
        let read = request.read_data(&mut buf)?;
        if read == 0 {
            break; // EOF
        }
        dest.write_all(&buf[..read])?;
    }

    Ok(())
}
