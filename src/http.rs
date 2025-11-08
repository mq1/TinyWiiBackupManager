// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;
use std::{
    fs::{self, File},
    io::{self, BufReader, BufWriter},
    path::Path,
    sync::LazyLock,
};
use tempfile::tempfile;
use ureq::tls::{RootCerts, TlsProvider};
use ureq::{Agent, tls::TlsConfig};
use zip::ZipArchive;

const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

#[cfg(feature = "native-tls")]
const TLS_PROVIDER: TlsProvider = TlsProvider::NativeTls;

#[cfg(feature = "native-tls")]
const ROOT_CERTS: RootCerts = RootCerts::PlatformVerifier;

#[cfg(feature = "bundled-tls")]
const TLS_PROVIDER: TlsProvider = TlsProvider::Rustls;

#[cfg(feature = "bundled-tls")]
const ROOT_CERTS: RootCerts = RootCerts::WebPki;

static AGENT: LazyLock<Agent> = LazyLock::new(|| {
    Agent::config_builder()
        .tls_config(
            TlsConfig::builder()
                .provider(TLS_PROVIDER)
                .root_certs(ROOT_CERTS)
                .build(),
        )
        .user_agent(USER_AGENT)
        .build()
        .into()
});

pub fn get(uri: &str) -> Result<Vec<u8>, ureq::Error> {
    AGENT.get(uri).call()?.body_mut().read_to_vec()
}

pub fn download_into_file(uri: &str, file: &File) -> Result<()> {
    let mut writer = BufWriter::new(file);

    let body = AGENT.get(uri).call()?.into_body();
    let mut reader = body.into_reader();
    io::copy(&mut reader, &mut writer)?;

    Ok(())
}

pub fn download_file(uri: &str, dest: &Path) -> Result<()> {
    let tmp = dest.with_extension("tmp");

    let res = {
        let tmp_file = File::create(&tmp)?;
        download_into_file(uri, &tmp_file)
    };

    if let Err(e) = res {
        fs::remove_file(&tmp)?;
        return Err(e);
    }

    fs::rename(&tmp, dest)?;

    Ok(())
}

pub fn download_and_extract_zip(uri: &str, dest_dir: &Path) -> Result<()> {
    let tmp = tempfile()?;

    download_into_file(uri, &tmp)?;

    let reader = BufReader::new(&tmp);
    let mut zip = ZipArchive::new(reader)?;
    zip.extract(dest_dir)?;

    Ok(())
}
