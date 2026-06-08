// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::game_id::GameID;
use anyhow::Result;
use nod::{
    common::{Compression, Format},
    write::FormatOptions,
};
use std::{
    ffi::OsStr,
    fs::{self, File},
    io,
    num::NonZeroUsize,
    path::Path,
    sync::LazyLock,
};
use ureq::tls::{RootCerts, TlsConfig, TlsProvider};
use zip::ZipArchive;

pub const SPLIT_SIZE: NonZeroUsize = NonZeroUsize::new(4_294_934_528).unwrap(); // 4 GiB - 32 KiB
pub const HEADER_SIZE: usize = 131_072; // 128 KiB
pub const BUF_SIZE: usize = 65_536; // 64 KiB

pub static AGENT: LazyLock<ureq::Agent> = LazyLock::new(|| {
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

pub fn get_threads_num() -> (usize, usize) {
    let cpus = num_cpus::get();

    let preloader_threads = match cpus {
        0..=4 => 1,
        5..=8 => 2,
        _ => 4,
    };

    let processor_threads = cpus - preloader_threads;

    (preloader_threads, processor_threads)
}

pub fn install_zips(
    root_dir: &Path,
    zips: impl IntoIterator<Item = impl AsRef<Path>>,
) -> Result<()> {
    for zip in zips {
        let mut f = File::open(zip)?;
        let mut archive = ZipArchive::new(&mut f)?;
        archive.extract(root_dir)?;
    }

    Ok(())
}

pub fn str_to_format(s: &str) -> Option<Format> {
    match s.to_ascii_lowercase().as_str() {
        "iso" | "gcm" => Some(Format::Iso),
        "ciso" => Some(Format::Ciso),
        "gcz" => Some(Format::Gcz),
        "nfs" => Some(Format::Nfs),
        "rvz" => Some(Format::Rvz),
        "wbfs" => Some(Format::Wbfs),
        "wia" => Some(Format::Wia),
        "tgc" => Some(Format::Tgc),
        _ => None,
    }
}

pub fn ext_to_format(ext: &OsStr) -> Option<Format> {
    ext.to_str().and_then(str_to_format)
}

pub fn format_to_opts(format: Format) -> FormatOptions {
    match format {
        Format::Rvz => FormatOptions {
            format: Format::Rvz,
            compression: Compression::Zstandard(19),
            block_size: Format::Rvz.default_block_size(),
        },
        format => FormatOptions::new(format),
    }
}

#[cfg(target_os = "macos")]
pub fn run_dot_clean(mount_point: &Path) -> Result<()> {
    let status = std::process::Command::new("dot_clean")
        .arg("-m")
        .arg(mount_point)
        .status()?;

    if !status.success() {
        anyhow::bail!("dot_clean failed with status {status}");
    }

    Ok(())
}

pub fn download_wiitdb_xml(root_dir: &Path) -> Result<()> {
    // Download wiitdb
    let mut resp = AGENT.get("https://www.gametdb.com/wiitdb.zip").call()?;
    let body = resp.body_mut().read_to_vec()?;

    // Open the archive
    let mut cursor = io::Cursor::new(body);
    let mut archive = ZipArchive::new(&mut cursor)?;
    let mut datafile = archive.by_name("wiitdb.xml")?;

    // Create the target directory.
    let target_dir = root_dir.join("apps").join("usbloader_gx");
    fs::create_dir_all(&target_dir)?;

    // Extract wiitdb.xml
    let target_path = target_dir.join("wiitdb.xml");
    let mut out_file = File::create(&target_path)?;
    io::copy(&mut datafile, &mut out_file)?;

    Ok(())
}

fn is_valid_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || " !#$%&'()+,-.;=@^_`{}~".contains(c)
}

pub fn sanitize_title(ascii_title: &str) -> String {
    let mut sanitized = String::with_capacity(64);

    let mut actual_title = false;
    for c in ascii_title.chars() {
        if sanitized.len() >= 64 {
            break;
        }

        if c.is_ascii_alphanumeric() {
            actual_title = true;
        }

        if actual_title && is_valid_char(c) {
            sanitized.push(c);
        }
    }

    if sanitized.is_empty() {
        sanitized.push_str("game");
    }

    let trimmed_len = sanitized.trim_end().len();
    sanitized.truncate(trimmed_len);

    sanitized
}

pub fn make_game_dir_name(game_id: GameID, fallback_title: &str) -> String {
    let ascii_title = twbm_idmap::get_ascii_title(game_id).unwrap_or(fallback_title);
    let sanitized_title = sanitize_title(&ascii_title);

    format!("{sanitized_title} [{game_id}]")
}
