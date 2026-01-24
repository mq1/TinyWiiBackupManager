// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{http_util, message::Message};
use anyhow::Result;
use iced::{Task, futures::TryFutureExt};
use semver::Version;
use std::sync::Arc;

const VERSION_URL: &str = concat!(
    env!("CARGO_PKG_REPOSITORY"),
    "/releases/latest/download/version.txt"
);

pub const LATEST_VERSION_DOWNLOAD_URL: &str =
    concat!(env!("CARGO_PKG_REPOSITORY"), "/releases/latest");

fn check() -> Result<Option<Version>> {
    let body = http_util::get_string(VERSION_URL)?;

    let latest_version = Version::parse(&body)?;
    let current_version = Version::parse(env!("CARGO_PKG_VERSION"))?;

    if latest_version > current_version {
        Ok(Some(latest_version))
    } else {
        Ok(None)
    }
}

pub fn get_check_update_task() -> Task<Message> {
    Task::perform(
        async { check() }.map_err(Arc::new),
        Message::GotLatestVersion,
    )
}
