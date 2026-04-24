// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::LazyLock;
use ureq::tls::{RootCerts, TlsConfig, TlsProvider};

const USER_AGENT: &str = concat!("TinyWiiBackupManager/", env!("CARGO_PKG_VERSION"));

#[cfg(all(feature = "native-tls", not(feature = "rustls")))]
const TLS_PROVIDER: TlsProvider = TlsProvider::NativeTls;

#[cfg(all(feature = "rustls", not(feature = "native-tls")))]
const TLS_PROVIDER: TlsProvider = TlsProvider::Rustls;

#[cfg(all(feature = "native-tls", not(feature = "rustls")))]
const ROOT_CERTS: RootCerts = RootCerts::PlatformVerifier;

#[cfg(all(feature = "rustls", not(feature = "native-tls")))]
const ROOT_CERTS: RootCerts = RootCerts::WebPki;

fn make_agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .user_agent(USER_AGENT)
        .tls_config(
            TlsConfig::builder()
                .provider(TLS_PROVIDER)
                .root_certs(ROOT_CERTS)
                .build(),
        )
        .build()
        .new_agent()
}

static AGENT: LazyLock<ureq::Agent> = LazyLock::new(make_agent);

pub fn get_vec(url: &str) -> Result<Vec<u8>, ureq::Error> {
    AGENT
        .get(url)
        .call()?
        .body_mut()
        .with_config()
        .limit(100 * 1024 * 1024)
        .read_to_vec()
}

pub fn get_string(url: &str) -> Result<String, ureq::Error> {
    AGENT.get(url).call()?.body_mut().read_to_string()
}
