// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::{Result, anyhow};
use std::{
    fs::File,
    io::{self, BufReader, BufWriter},
    path::Path,
    sync::LazyLock,
};
use tempfile::{NamedTempFile, tempfile};
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

pub fn get_string(uri: &str) -> Result<String, ureq::Error> {
    AGENT.get(uri).call()?.body_mut().read_to_string()
}

pub fn send_form<I: IntoIterator<Item = (S, S)>, S: AsRef<str>>(
    uri: &str,
    form: I,
) -> Result<Vec<u8>, ureq::Error> {
    AGENT.post(uri).send_form(form)?.body_mut().read_to_vec()
}

pub fn download_into_file(uri: &str, file: &File) -> Result<()> {
    let mut writer = BufWriter::new(file);

    let mut resp = AGENT.get(uri).call()?;
    let body = resp.body_mut();
    let mut reader = body.as_reader();
    io::copy(&mut reader, &mut writer)?;

    Ok(())
}

pub fn download_file(uri: &str, dest: &Path) -> Result<()> {
    let parent = dest.parent().ok_or(anyhow!("No parent directory"))?;
    let tmp = NamedTempFile::new_in(parent)?;

    download_into_file(uri, tmp.as_file())?;
    tmp.persist(dest)?;

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
