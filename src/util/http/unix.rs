// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::util::http::USER_AGENT;
use anyhow::Result;
use std::sync::LazyLock;
use ureq::{
    Agent,
    tls::{RootCerts, TlsConfig, TlsProvider},
};

#[cfg(feature = "native-https")]
const PROVIDER: TlsProvider = TlsProvider::NativeTls;

#[cfg(feature = "static-https")]
const PROVIDER: TlsProvider = TlsProvider::Rustls;

static AGENT: LazyLock<Agent> = LazyLock::new(|| {
    Agent::config_builder()
        .user_agent(USER_AGENT)
        .tls_config(
            TlsConfig::builder()
                .provider(PROVIDER)
                .root_certs(RootCerts::PlatformVerifier)
                .build(),
        )
        .build()
        .new_agent()
});

pub fn download(url: &str, mut dest: impl std::io::Write) -> Result<()> {
    let mut resp = AGENT.get(url).call()?;
    let mut body = resp.body_mut().as_reader();

    std::io::copy(&mut body, &mut dest)?;

    Ok(())
}
