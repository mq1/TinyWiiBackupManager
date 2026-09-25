// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::util::http::USER_AGENT;
use anyhow::Result;
use curl::easy::{Easy, WriteError};

pub fn download(url: &str, mut dest: impl std::io::Write) -> Result<()> {
    let mut easy = Easy::new();
    easy.useragent(USER_AGENT)?;
    easy.fail_on_error(true)?;
    easy.url(url)?;

    let mut transfer = easy.transfer();
    transfer.write_function(move |data| {
        dest.write_all(data).map_err(|_| WriteError::Pause)?;
        Ok(data.len())
    })?;
    transfer.perform()?;

    Ok(())
}

pub fn post_then_download(url: &str, body: &[u8], mut dest: impl std::io::Write) -> Result<()> {
    let mut easy = Easy::new();
    easy.useragent(USER_AGENT)?;
    easy.fail_on_error(true)?;
    easy.url(url)?;
    easy.post(true)?;
    easy.post_fields_copy(body)?;

    let mut transfer = easy.transfer();
    transfer.write_function(move |data| {
        dest.write_all(data).map_err(|_| WriteError::Pause)?;
        Ok(data.len())
    })?;
    transfer.perform()?;

    Ok(())
}
