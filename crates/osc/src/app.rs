// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;
use jiff::Timestamp;
use serde::Deserialize;
use size::Size;
use std::{
    hash::{Hash, Hasher},
    path::Path,
};
use twbm_common::util::make_search_term;
use twbm_http::{download_and_extract_zip, download_and_send_via_wiiload, download_file};

fn make_osc_url(slug: &str) -> Box<str> {
    const BASE: &str = "https://oscwii.org/library/app/";

    let mut url = String::with_capacity(BASE.len() + slug.len());
    url.push_str(BASE);
    url.push_str(slug);

    url.into()
}

#[derive(Debug, Clone, Deserialize)]
pub struct OscAppAsset {
    url: Box<str>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OscAppAssets {
    icon: OscAppAsset,
    archive: OscAppAsset,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OscAppDescription {
    short: Box<str>,
    long: Box<str>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OscAppMeta {
    slug: Box<str>,
    name: Box<str>,
    author: Box<str>,
    version: Box<str>,
    assets: OscAppAssets,
    uncompressed_size: Size,
    description: OscAppDescription,

    #[serde(with = "jiff::fmt::serde::timestamp::second::required")]
    release_date: Timestamp,
}

#[derive(Debug, Clone)]
pub struct OscApp {
    meta: OscAppMeta,
    icon: Option<Box<Path>>,
    search_term_lowercase: Box<str>,
    osc_url: Box<str>,
}

impl<'de> serde::Deserialize<'de> for OscApp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let meta = OscAppMeta::deserialize(deserializer)?;

        let search_term_lowercase = make_search_term(&[&meta.name, &meta.slug]);
        let osc_url = make_osc_url(&meta.slug);

        Ok(OscApp {
            meta,
            icon: None,
            search_term_lowercase,
            osc_url,
        })
    }
}

impl OscApp {
    pub fn download_icon(&self, data_dir: &Path) -> Result<bool> {
        let icon_path = data_dir
            .join("osc-icons")
            .join(&*self.meta.slug)
            .with_added_extension("png");

        download_file(&self.meta.assets.icon.url, &icon_path)
    }

    pub fn set_icon(&mut self, data_dir: &Path) {
        let icon_path = data_dir
            .join("osc-icons")
            .join(&*self.meta.slug)
            .with_added_extension("png");

        self.icon = icon_path.exists().then(|| icon_path.into())
    }

    pub fn install(&self, root_dir: &Path) -> Result<()> {
        download_and_extract_zip(&self.meta.assets.archive.url, root_dir)
    }

    pub fn wiiload(&self, wii_ip: &str) -> Result<Vec<String>> {
        download_and_send_via_wiiload(&self.meta.assets.archive.url, wii_ip)
    }

    pub fn version(&self) -> &str {
        &self.meta.version
    }

    pub fn size(&self) -> Size {
        self.meta.uncompressed_size
    }

    pub fn name(&self) -> &str {
        &self.meta.name
    }

    pub fn matches_search(&self, search_term: &str) -> bool {
        self.search_term_lowercase.contains(search_term)
    }

    pub fn icon_uri(&self) -> &str {
        &self.meta.assets.icon.url
    }

    pub fn icon(&self) -> Option<&Path> {
        self.icon.as_deref()
    }

    pub fn slug(&self) -> &str {
        &self.meta.slug
    }

    pub fn release_date(&self) -> Timestamp {
        self.meta.release_date
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

impl Hash for OscApp {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.meta.slug.hash(state);
    }
}
