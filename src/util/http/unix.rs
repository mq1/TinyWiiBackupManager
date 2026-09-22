// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::util::http::USER_AGENT;
use anyhow::Result;
use curl::easy::{Easy, WriteError};

pub fn download(url: &str, mut dest: impl std::io::Write) -> Result<()> {
    let mut easy = Easy::new();
    easy.useragent(USER_AGENT)?;
    easy.url(url)?;

    let mut transfer = easy.transfer();
    transfer.write_function(move |data| {
        dest.write_all(data).map_err(|_| WriteError::Pause)?;
        Ok(data.len())
    })?;
    transfer.perform()?;

    Ok(())
}
