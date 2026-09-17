// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{errors::Error, homebrew::meta::HomebrewAppMeta, util::fs::get_dir_size};
use iced::{
    advanced::image::{Allocation, allocate},
    widget::image,
};
use size::Size;
use smol::fs;
use smol_str::{SmolStr, StrExt, ToSmolStr, format_smolstr};
use std::{
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
};

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
    size: Size,
    size_str: SmolStr,
    icon: Option<Allocation>,
    osc_url: OsString,
    search_term_lowercase: SmolStr,
}

impl HomebrewApp {
    pub async fn try_from_path(path: impl Into<PathBuf>) -> Result<Self, Error> {
        let path = path.into();

        // Check if the path is a directory
        if !fs::metadata(&path).await?.is_dir() {
            return Err(Error::NotADir);
        }

        // Get the directory name
        let dir_name = path
            .file_name()
            .and_then(OsStr::to_str)
            .ok_or(Error::InvalidFilename)?;

        if dir_name.starts_with('.') {
            return Err(Error::HiddenDir);
        }

        let meta = HomebrewAppMeta::parse(&path)?;

        let size = get_dir_size(&path).await;
        let size_str = size.to_smolstr();

        let icon = {
            let path = path.join("icon.png");

            fs::read(&path)
                .await
                .ok()
                .map(image::Handle::from_bytes)
                .map(|handle| unsafe { allocate(&handle, (128, 48).into()) })
        };

        let osc_url = make_osc_url(&path);

        let search_term_lowercase =
            format_smolstr!("{}\0{}", meta.name(), dir_name).to_lowercase_smolstr();

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

    pub fn size(&self) -> Size {
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
