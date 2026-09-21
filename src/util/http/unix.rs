// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::util::http::USER_AGENT;
use anyhow::{Result, bail};

pub fn download(url: &str, mut dest: impl std::io::Write) -> Result<()> {
    let mut resp = minreq::get(url)
        .with_header("User-Agent", USER_AGENT)
        .send_lazy()?;

    if resp.status_code != 200 {
        bail!("HTTP error: {}", resp.status_code);
    }

    std::io::copy(&mut resp, &mut dest)?;

    Ok(())
}
