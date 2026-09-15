// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    errors::Error,
    util::http::{download_and_extract_zip, download_and_send_via_wiiload, download_file},
};
use serde::Deserialize;
use smol_str::SmolStr;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct OscAppAsset {
    pub url: SmolStr,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OscAppAssets {
    icon: OscAppAsset,
    archive: OscAppAsset,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OscAppDescription {
    short: SmolStr,
    long: SmolStr,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OscApp {
    slug: SmolStr,
    name: SmolStr,
    author: SmolStr,
    version: SmolStr,
    assets: OscAppAssets,
    uncompressed_size: u64,
    release_date: i64,
    description: OscAppDescription,
}

impl OscApp {
    pub async fn download_icon(&self, data_dir: &Path) -> Result<(), Error> {
        let icon_path = data_dir
            .join("osc-icons")
            .join(&self.slug)
            .with_added_extension("png");

        download_file(&self.assets.icon.url, &icon_path).await
    }

    pub async fn install(&self, root_dir: &Path) -> Result<(), Error> {
        download_and_extract_zip(&self.assets.archive.url, root_dir).await
    }

    pub async fn wiiload(&self, wii_ip: &str) -> Result<(), Error> {
        download_and_send_via_wiiload(&self.assets.archive.url, wii_ip).await
    }
}
