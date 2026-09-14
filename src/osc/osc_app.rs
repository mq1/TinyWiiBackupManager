// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    errors::Error,
    util::http::{download_and_extract_zip, download_and_send_via_wiiload, download_file},
};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct OscAppMetaAsset {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OscAppMetaAssets {
    icon: OscAppMetaAsset,
    archive: OscAppMetaAsset,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OscAppMetaDescription {
    pub short: String,
    pub long: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OscAppMeta {
    pub slug: String,
    pub name: String,
    pub author: String,
    pub version: String,
    pub assets: OscAppMetaAssets,
    pub uncompressed_size: u64,
    pub release_date: i64,
    pub description: OscAppMetaDescription,
}

#[derive(Debug, Clone)]
pub struct OscApp {
    pub meta: OscAppMeta,
    pub search_term: String,
}

impl OscApp {
    pub async fn download_icon(&self, data_dir: &Path) -> Result<(), Error> {
        let icon_path = data_dir
            .join("osc-icons")
            .join(&self.meta.slug)
            .with_added_extension("png");

        download_file(&self.meta.assets.icon.url, &icon_path).await
    }

    pub async fn install(&self, root_dir: &Path) -> Result<(), Error> {
        download_and_extract_zip(&self.meta.assets.archive.url, root_dir).await
    }

    pub async fn wiiload(&self, wii_ip: &str) -> Result<(), Error> {
        download_and_send_via_wiiload(&self.meta.assets.archive.url, wii_ip).await
    }
}
