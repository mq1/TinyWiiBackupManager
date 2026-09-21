// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::util::http::USER_AGENT;
use anyhow::{Result, bail};
use winsafe::{HINTERNET, HttpInfo, co};

pub fn download(url: &str, mut dest: impl std::io::Write) -> Result<()> {
    let inet = HINTERNET::InternetOpen(
        USER_AGENT,
        co::INTERNET_OPEN_TYPE::PRECONFIG,
        None, // proxy name (unused with PRECONFIG)
        None, // proxy bypass
        co::INTERNET_FLAG::NoValue,
    )?;

    let req = inet.InternetOpenUrl(
        url,
        None,                                                  // extra request headers
        co::INTERNET_FLAG::SECURE | co::INTERNET_FLAG::RELOAD, // RELOAD = skip cache
        None,                                                  // context
    )?;

    let status = req.HttpQueryInfo(co::HTTP_QUERY::STATUS_CODE, co::HTTP_QUERY_FLAG::NUMBER)?;
    if status.unwrap_number() != 200 {
        bail!("HTTP error: {status}");
    }

    let mut buf = [0u8; 64 * 1024];
    loop {
        let read = req.InternetReadFile(&mut buf)? as usize;
        if read == 0 {
            break; // EOF
        }
        dest.write_all(&buf[..read])?;
    }

    Ok(())
}
