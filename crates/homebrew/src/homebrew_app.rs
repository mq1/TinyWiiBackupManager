// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::homebrew_app_meta::HomebrewAppMeta;
use anyhow::{Context, Result, bail};
use size::Size;
use std::{
    ffi::OsStr,
    fs,
    hash::{Hash, Hasher},
    path::Path,
};

fn make_osc_url(dirname: &str) -> Box<str> {
    const BASE: &str = "https://oscwii.org/library/app/";

    let mut url = String::with_capacity(BASE.len() + dirname.len());
    url.push_str(BASE);
    url.push_str(dirname);

    url.into()
}

fn make_search_term(app_name: &str, dirname: &str) -> Box<str> {
    let mut term = String::with_capacity(app_name.len() + 1 + dirname.len());
    term.push_str(app_name);
    term.push('\0');
    term.push_str(dirname);

    if term.is_ascii() {
        term.make_ascii_lowercase();
    } else {
        term = term.to_lowercase();
    };

    term.into()
}

#[derive(Debug, Clone)]
pub struct HomebrewApp {
    path: Box<Path>,
    dirname: Box<str>,
    meta: HomebrewAppMeta,
    size: Option<Size>,
    icon: Option<Box<Path>>,
    osc_url: Box<str>,
    search_term_lowercase: Box<str>,
}

impl HomebrewApp {
    pub fn try_from_path(path: impl Into<Box<Path>>) -> Result<Self> {
        let path = path.into();

        // Check if the path is a directory
        if !fs::metadata(&path)?.is_dir() {
            bail!("{} is not a directory", path.display());
        }

        // Get the directory name
        let dirname = path
            .file_name()
            .and_then(OsStr::to_str)
            .context("invalid filename")?;

        if dirname.starts_with('.') {
            bail!("hidden directory");
        }

        let meta = HomebrewAppMeta::parse(&path)?;

        let size = fs_extra::dir::get_size(&path).ok().map(Size::from_bytes);

        let icon_path = path.join("icon.png");
        let icon = fs::exists(&icon_path)
            .unwrap_or(false)
            .then(|| icon_path.into());

        let osc_url = make_osc_url(dirname);
        let search_term_lowercase = make_search_term(meta.name(), dirname);

        Ok(Self {
            dirname: dirname.into(),
            path,
            meta,
            size,
            icon,
            osc_url,
            search_term_lowercase,
        })
    }

    pub fn matches_search(&self, search_term: &str) -> bool {
        self.search_term_lowercase.contains(search_term)
    }

    pub fn size(&self) -> Option<Size> {
        self.size
    }

    pub fn meta(&self) -> &HomebrewAppMeta {
        &self.meta
    }

    pub fn icon(&self) -> Option<&Path> {
        self.icon.as_deref()
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn dirname(&self) -> &str {
        &self.dirname
    }

    pub fn osc_url(&self) -> &str {
        &self.osc_url
    }
}

impl PartialEq for HomebrewApp {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl Eq for HomebrewApp {}

impl Hash for HomebrewApp {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.path.hash(state)
    }
}
