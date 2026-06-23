// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;
use std::{fs, path::Path, sync::LazyLock};
use ureq::tls::{RootCerts, TlsConfig, TlsProvider};

const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

static AGENT: LazyLock<ureq::Agent> = LazyLock::new(|| {
    const USER_AGENT: &str = concat!("TinyWiiBackupManager/", env!("CARGO_PKG_VERSION"));

    #[cfg(feature = "native-tls")]
    const PROVIDER: TlsProvider = TlsProvider::NativeTls;

    #[cfg(feature = "rustls")]
    const PROVIDER: TlsProvider = TlsProvider::Rustls;

    ureq::Agent::config_builder()
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

pub fn download_file(uri: &str, dest: impl AsRef<Path>) -> Result<()> {
    let bytes = AGENT
        .get(uri)
        .call()?
        .body_mut()
        .with_config()
        .limit(100 * 1024 * 1024)
        .read_to_vec()?;

    fs::write(dest, bytes)?;

    Ok(())
}
