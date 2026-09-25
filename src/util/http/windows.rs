// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::util::http::USER_AGENT;
use anyhow::{Result, bail};
use winsafe::{HINTERNET, HttpInfo, SysResult, co, guard::InternetCloseHandleGuard};

fn inet() -> SysResult<InternetCloseHandleGuard<HINTERNET>> {
    HINTERNET::InternetOpen(
        USER_AGENT,
        co::INTERNET_OPEN_TYPE::PRECONFIG,
        None,
        None,
        co::INTERNET_FLAG::NoValue,
    )
}

pub fn download(url: &str, mut dest: impl std::io::Write) -> Result<()> {
    let inet = inet()?;

    let req = inet.InternetOpenUrl(
        url,
        None,                                                  // extra request headers
        co::INTERNET_FLAG::SECURE | co::INTERNET_FLAG::RELOAD, // RELOAD = skip cache
        None,                                                  // context
    )?;

    match req.HttpQueryInfo(co::HTTP_QUERY::STATUS_CODE, co::HTTP_QUERY_FLAG::NUMBER)? {
        HttpInfo::Number(status) if status >= 400 => bail!("HTTP error: {status}"),
        HttpInfo::Number(_) => {}
        _ => bail!("HTTP error: {status}"),
    };

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

pub fn post_then_download(url: &str, body: &[u8], mut dest: impl std::io::Write) -> Result<()> {
    let inet = inet()?;

    let Some(rest) = url.strip_prefix("https://") else {
        bail!("invalid URL: {url}");
    };

    let (host, path) = match rest.find('/') {
        Some(idx) => (&rest[..idx], &rest[idx..]),
        None => (rest, "/"),
    };

    let conn = inet.InternetConnect(
        host,
        co::INTERNET_DEFAULT_PORT::HTTPS,
        None, // user_name
        None, // password
        co::INTERNET_SERVICE::HTTP,
        co::INTERNET_FLAG::NoValue,
        None, // context
    )?;

    let req = conn.HttpOpenRequest(
        Some("POST"),
        path,
        None,           // version (defaults to HTTP/1.1)
        None,           // referrer
        &[] as &[&str], // accept_types
        co::INTERNET_FLAG::SECURE | co::INTERNET_FLAG::RELOAD,
        None, // context
    )?;

    req.HttpSendRequest(
        Some("Content-Type: application/x-www-form-urlencoded"), // headers
        body,
    )?;

    match req.HttpQueryInfo(co::HTTP_QUERY::STATUS_CODE, co::HTTP_QUERY_FLAG::NUMBER)? {
        HttpInfo::Number(status) if status >= 400 => bail!("HTTP error: {status}"),
        HttpInfo::Number(_) => {}
        _ => bail!("HTTP error: {status}"),
    };

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
