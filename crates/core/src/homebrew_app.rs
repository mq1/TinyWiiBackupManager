// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use image::{ImageFormat, RgbaImage};
use serde::Deserialize;
use std::{
    cmp::Ordering,
    fs,
    path::{Path, PathBuf},
};

use crate::config::SortBy;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct HomebrewAppMeta {
    pub name: String,

    #[serde(default)]
    pub coder: String,

    #[serde(default)]
    pub version: String,

    #[serde(default, deserialize_with = "deser_date")]
    pub release_date: String,

    #[serde(default)]
    pub short_description: String,

    #[serde(default)]
    pub long_description: String,
}

#[derive(Debug, Clone)]
pub struct HomebrewApp {
    pub path: PathBuf,
    pub meta: HomebrewAppMeta,
    pub size: u64,
    pub icon_rgba8: RgbaImage,
    pub search_term: String,
}

impl HomebrewApp {
    pub fn from_path(path: PathBuf) -> Option<Self> {
        let xml = fs::read_to_string(path.join("meta.xml")).ok()?;
        let mut meta = quick_xml::de::from_str::<HomebrewAppMeta>(&xml).ok()?;

        if let Some(name) = meta.name.strip_prefix(' ') {
            meta.name = name.to_string();
        }

        let size = fs_extra::dir::get_size(&path).ok()?;

        let icon_bytes = fs::read(path.join("icon.png")).unwrap_or_default();
        let icon =
            image::load_from_memory_with_format(&icon_bytes, ImageFormat::Png).unwrap_or_default();
        let icon_rgba8 = icon.into_rgba8();

        let search_term = format!("{}\0{}", &meta.name, &meta.coder).to_lowercase();

        Some(Self {
            path,
            meta,
            size,
            icon_rgba8,
            search_term,
        })
    }
}

fn deser_date<'de, D>(d: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let mut s = String::deserialize(d)?;

    if s.len() >= 8 {
        let year = &s[0..4];
        let month = &s[4..6];
        let day = &s[6..8];

        s = format!("{year}-{month}-{day}");
    }

    Ok(s)
}

pub fn scan_dir(path: &Path) -> Vec<HomebrewApp> {
    let Ok(entries) = fs::read_dir(path) else {
        return Vec::new();
    };

    entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            HomebrewApp::from_path(entry.path())
        })
        .collect()
}

pub fn get_compare_fn(sort_by: SortBy) -> impl FnMut(&HomebrewApp, &HomebrewApp) -> Ordering {
    move |a, b| match sort_by {
        SortBy::NameDescending => a.meta.name.cmp(&b.meta.name),
        SortBy::NameAscending => b.meta.name.cmp(&a.meta.name),
        SortBy::SizeDescending => a.size.cmp(&b.size),
        SortBy::SizeAscending => b.size.cmp(&a.size),
    }
}
