// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::errors::Error;
use smol::fs;
use std::{ffi::OsStr, path::Path, sync::LazyLock};
use ureq::{
    Agent,
    tls::{RootCerts, TlsConfig, TlsProvider},
};

const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

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

/// Downloads a file, creating the parent directory if needed
/// Skips if the file already exists
pub async fn download_file(uri: &str, dest: impl AsRef<Path>) -> Result<(), Error> {
    let dest = dest.as_ref();

    let dest_filename = dest
        .file_name()
        .and_then(OsStr::to_str)
        .ok_or(Error::InvalidFilename)?;

    if fs::symlink_metadata(&dest).await.is_ok() {
        println!("INFO: {} already exists, skipping", dest.display());
        return Ok(());
    }

    let dest_parent = dest.parent().ok_or(Error::InvalidFilename)?;
    fs::create_dir_all(dest_parent).await?;

    smol::unblock({
        let uri = uri.to_string();
        let dest = dest.to_path_buf();
        let dest_filename = dest_filename.to_string();
        let dest_parent = dest_parent.to_path_buf();

        move || {
            let mut resp = AGENT.get(uri).call()?;
            let mut body = resp.body_mut().as_reader();

            let mut out = tempfile::Builder::new()
                .prefix(&dest_filename)
                .suffix(".part")
                .rand_bytes(0)
                .tempfile_in(dest_parent)?;

            std::io::copy(&mut body, &mut out)?;
            out.persist(&dest)?;

            Ok(())
        }
    })
    .await
}

pub async fn download_file_with_fallback(
    uri: &str,
    dest: impl AsRef<Path>,
    fallback: &str,
) -> Result<(), Error> {
    if download_file(uri, &dest).await.is_err() {
        download_file(fallback, dest).await
    } else {
        Ok(())
    }
}
