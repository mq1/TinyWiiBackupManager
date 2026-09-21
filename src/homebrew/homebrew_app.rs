// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{homebrew::meta::HomebrewAppMeta, util::fs::get_dir_size};
use anyhow::{Context, Result, bail};
use compact_str::{CompactString, ToCompactString, format_compact};
use iced::{
    advanced::image::{Allocation, allocate},
    widget::image,
};
use size::Size;
use smol::fs;
use std::{
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
};
use tap::Pipe;

pub fn make_osc_url(path: &Path) -> OsString {
    let mut base = OsString::from("https://oscwii.org/library/app/");

    if let Some(slug) = path.file_name() {
        base.push(slug);
    }

    base
}

#[derive(Debug, Clone)]
pub struct HomebrewApp {
    path: PathBuf,
    meta: HomebrewAppMeta,
    size: u64,
    size_str: CompactString,
    icon: Option<Allocation>,
    osc_url: OsString,
    search_term_lowercase: CompactString,
}

impl HomebrewApp {
    pub async fn try_from_path(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();

        // Check if the path is a directory
        if !fs::metadata(&path).await?.is_dir() {
            bail!("{} is not a directory", path.display());
        }

        // Get the directory name
        let dir_name = path
            .file_name()
            .and_then(OsStr::to_str)
            .context("invalid filename")?;

        if dir_name.starts_with('.') {
            bail!("hidden directory");
        }

        let meta = HomebrewAppMeta::parse(&path)?;

        let size = get_dir_size(&path).await;
        let size_str = Size::from_bytes(size).to_compact_string();

        let icon = path
            .join("icon.png")
            .pipe(fs::read)
            .await
            .ok()
            .map(image::Handle::from_bytes)
            .map(|handle| unsafe { allocate(&handle, (128, 48).into()) });

        let osc_url = make_osc_url(&path);

        let search_term_lowercase = format_compact!("{}\0{}", meta.name(), dir_name).to_lowercase();

        Ok(Self {
            path,
            meta,
            size,
            size_str,
            icon,
            osc_url,
            search_term_lowercase,
        })
    }

    pub fn matches_search(&self, search_term: &str) -> bool {
        self.search_term_lowercase.contains(search_term)
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn size_str(&self) -> &str {
        &self.size_str
    }

    pub fn name(&self) -> &str {
        self.meta.name()
    }

    pub fn version(&self) -> &str {
        self.meta.version()
    }

    pub fn release_date(&self) -> &str {
        self.meta.release_date()
    }

    pub fn coder(&self) -> &str {
        self.meta.coder()
    }

    pub fn short_description(&self) -> &str {
        self.meta.short_description()
    }

    pub fn long_description(&self) -> &str {
        self.meta.long_description()
    }

    pub fn icon(&self) -> Option<&image::Handle> {
        self.icon.as_ref().map(|icon| icon.handle())
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn osc_url(&self) -> &OsStr {
        &self.osc_url
    }
}

impl PartialEq for HomebrewApp {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl Eq for HomebrewApp {}
