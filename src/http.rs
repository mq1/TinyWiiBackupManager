// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use std::convert::Into;
use std::sync::LazyLock;
use ureq::Agent;
use ureq::tls::{RootCerts, TlsConfig, TlsProvider};

const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

#[cfg(feature = "native-tls")]
const TLS_PROVIDER: TlsProvider = TlsProvider::NativeTls;

#[cfg(feature = "native-tls")]
const ROOT_CERTS: RootCerts = RootCerts::PlatformVerifier;

#[cfg(feature = "bundled-tls")]
const TLS_PROVIDER: TlsProvider = TlsProvider::Rustls;

#[cfg(feature = "bundled-tls")]
const ROOT_CERTS: RootCerts = RootCerts::WebPki;

pub static AGENT: LazyLock<Agent> = LazyLock::new(|| {
    Agent::config_builder()
        .tls_config(
            TlsConfig::builder()
                .provider(TLS_PROVIDER)
                .root_certs(ROOT_CERTS)
                .build(),
        )
        //.timeout_global(Some(Duration::from_secs(30)))
        .user_agent(USER_AGENT)
        .build()
        .into()
});
