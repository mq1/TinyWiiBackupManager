// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::util::http::{download_and_extract_zip, download_and_send_via_wiiload, download_file};
use anyhow::Result;
use compact_str::{CompactString, ToCompactString, format_compact};
use iced::{
    advanced::image::{Allocation, allocate},
    widget::image::{self, Handle},
};
use serde::Deserialize;
use size::Size;
use std::path::Path;
use tap::Pipe;
use time::OffsetDateTime;

#[derive(Debug, Clone, Deserialize)]
pub struct OscAppAsset {
    url: CompactString,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OscAppAssets {
    icon: OscAppAsset,
    archive: OscAppAsset,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OscAppDescription {
    short: CompactString,
    long: CompactString,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OscAppMeta {
    slug: CompactString,
    name: CompactString,
    author: CompactString,
    version: CompactString,
    assets: OscAppAssets,
    uncompressed_size: u64,
    release_date: i64,
    description: OscAppDescription,
}

#[derive(Debug, Clone)]
pub struct OscApp {
    meta: OscAppMeta,
    icon: Option<Allocation>,
    uncompressed_size_str: CompactString,
    search_term_lowercase: CompactString,
    release_date_str: CompactString,
    osc_url: CompactString,
}

impl<'de> serde::Deserialize<'de> for OscApp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let meta = OscAppMeta::deserialize(deserializer)?;

        let uncompressed_size_str = Size::from_bytes(meta.uncompressed_size).to_compact_string();
        let search_term_lowercase = format_compact!("{}\0{}", meta.name, meta.slug).to_lowercase();

        let release_date_str = meta
            .release_date
            .pipe(OffsetDateTime::from_unix_timestamp)
            .map_err(|_| serde::de::Error::custom("invalid release date"))
            .map(|dt| format_compact!("{}-{}-{}", dt.year(), dt.month(), dt.day()))?;

        let osc_url = format_compact!("https://oscwii.org/library/app/{}", meta.slug);

        Ok(OscApp {
            meta,
            icon: None,
            uncompressed_size_str,
            search_term_lowercase,
            release_date_str,
            osc_url,
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

    pub fn load_icon_blocking(&mut self, data_dir: &Path) {
        self.icon = data_dir
            .join("osc-icons")
            .join(&self.meta.slug)
            .with_added_extension("png")
            .pipe(std::fs::read)
            .ok()
            .map(image::Handle::from_bytes)
            .map(|handle| unsafe { allocate(&handle, (128, 48).into()) });
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

    pub fn slug_and_icon_url(&self) -> (CompactString, CompactString) {
        (self.meta.slug.clone(), self.meta.assets.icon.url.clone())
    }

    pub fn icon(&self) -> Option<&Handle> {
        self.icon.as_ref().map(|icon| icon.handle())
    }

    pub fn slug(&self) -> &str {
        &self.meta.slug
    }

    pub fn release_date_str(&self) -> &str {
        &self.release_date_str
    }

    pub fn coder(&self) -> &str {
        &self.meta.author
    }

    pub fn short_description(&self) -> &str {
        &self.meta.description.short
    }

    pub fn long_description(&self) -> &str {
        &self.meta.description.long
    }

    pub fn osc_url(&self) -> &str {
        &self.osc_url
    }
}
