// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::util::http::{download_and_extract_zip, download_and_send_via_wiiload, download_file};
use anyhow::Result;
use iced::{advanced::image::Allocation, widget::image::Handle};
use serde::Deserialize;
use size::Size;
use smol_str::{SmolStr, StrExt, ToSmolStr, format_smolstr};
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
pub struct OscAppMeta {
    slug: SmolStr,
    name: SmolStr,
    author: SmolStr,
    version: SmolStr,
    assets: OscAppAssets,
    uncompressed_size: u64,
    release_date: i64,
    description: OscAppDescription,
}

#[derive(Debug, Clone)]
pub struct OscApp {
    meta: OscAppMeta,
    icon: Option<Allocation>,
    uncompressed_size_str: SmolStr,
    search_term_lowercase: SmolStr,
}

impl<'de> serde::Deserialize<'de> for OscApp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        OscAppMeta::deserialize(deserializer).map(|meta| {
            let uncompressed_size_str = Size::from_bytes(meta.uncompressed_size).to_smolstr();
            let search_term_lowercase =
                format_smolstr!("{}\0{}", meta.name, meta.slug).to_lowercase_smolstr();

            OscApp {
                meta,
                icon: None,
                uncompressed_size_str,
                search_term_lowercase,
            }
        })
    }
}

impl OscApp {
    pub async fn download_icon(&self, data_dir: &Path) -> Result<()> {
        let icon_path = data_dir
            .join("osc-icons")
            .join(&self.meta.slug)
            .with_added_extension("png");

        download_file(&self.meta.assets.icon.url, &icon_path).await
    }

    pub async fn install(&self, root_dir: &Path) -> Result<()> {
        download_and_extract_zip(&self.meta.assets.archive.url, root_dir).await
    }

    pub async fn wiiload(&self, wii_ip: &str) -> Result<()> {
        download_and_send_via_wiiload(&self.meta.assets.archive.url, wii_ip).await
    }

    pub fn version(&self) -> &str {
        &self.meta.version
    }

    pub fn size(&self) -> u64 {
        self.meta.uncompressed_size
    }

    pub fn size_str(&self) -> &str {
        &self.uncompressed_size_str
    }

    pub fn name(&self) -> &str {
        &self.meta.name
    }

    pub fn matches_search(&self, search_term: &str) -> bool {
        self.search_term_lowercase.contains(search_term)
    }

    pub fn icon(&self) -> Option<&Handle> {
        self.icon.as_ref().map(|icon| icon.handle())
    }
}
