// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use std::convert::Into;
use std::sync::LazyLock;
use std::time::Duration;
use ureq::Agent;
use ureq::tls::{RootCerts, TlsConfig, TlsProvider};

const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

#[cfg(any(
    target_os = "macos",
    all(
        target_os = "windows",
        any(target_arch = "x86_64", target_arch = "aarch64")
    )
))]
const TLS_PROVIDER: TlsProvider = TlsProvider::NativeTls;

#[cfg(any(
    target_os = "macos",
    all(
        target_os = "windows",
        any(target_arch = "x86_64", target_arch = "aarch64")
    )
))]
const ROOT_CERTS: RootCerts = RootCerts::PlatformVerifier;

#[cfg(not(any(
    target_os = "macos",
    all(
        target_os = "windows",
        any(target_arch = "x86_64", target_arch = "aarch64")
    )
)))]
const TLS_PROVIDER: TlsProvider = TlsProvider::Rustls;

#[cfg(not(any(
    target_os = "macos",
    all(
        target_os = "windows",
        any(target_arch = "x86_64", target_arch = "aarch64")
    )
)))]
const ROOT_CERTS: RootCerts = RootCerts::WebPki;

pub static AGENT: LazyLock<Agent> = LazyLock::new(|| {
    Agent::config_builder()
        .tls_config(
            TlsConfig::builder()
                .provider(TLS_PROVIDER)
                .root_certs(ROOT_CERTS)
                .build(),
        )
        .timeout_global(Some(Duration::from_secs(10)))
        .user_agent(USER_AGENT)
        .build()
        .into()
});
