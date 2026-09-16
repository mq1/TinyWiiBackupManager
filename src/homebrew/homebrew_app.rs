// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{errors::Error, homebrew::meta::HomebrewAppMeta, util::fs::get_dir_size};
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

pub fn make_osc_url(path: &Path) -> OsString {
    let mut base = OsString::from("https://oscwii.org/library/app/");

    if let Some(slug) = path.file_name() {
        base.push(slug);
    }

    base
}

#[derive(Debug, Clone)]
pub struct HomebrewApp {
    pub path: PathBuf,
    pub meta: HomebrewAppMeta,
    pub size: Size,
    pub icon: Option<Allocation>,
    pub osc_url: OsString,
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

        let icon = {
            let path = path.join("icon.png");

            fs::read(&path)
                .await
                .ok()
                .map(image::Handle::from_bytes)
                .map(|handle| unsafe { allocate(&handle, (128, 48).into()) })
        };

        let osc_url = make_osc_url(&path);

        Ok(Self {
            path,
            meta,
            size,
            icon,
            osc_url,
        })
    }
}

impl PartialEq for HomebrewApp {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl Eq for HomebrewApp {}
