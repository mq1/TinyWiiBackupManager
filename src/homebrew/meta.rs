// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::errors::Error;
use std::{fs, path::PathBuf};

fn get_value(contents: &str, start_pattern: &str, end_pattern: &str) -> Option<String> {
    let start = contents.find(start_pattern)?;
    let slice = &contents[start..];
    let start = slice.find('>')? + 1;
    let slice = &slice[start..];
    let end = slice.find(end_pattern)?;
    let value = &slice[..end];

    Some(value.trim().to_string())
}

macro_rules! get_property {
    ($contents:expr, $element:literal) => {{
        const START_PATTERN: &str = concat!("<", $element);
        const END_PATTERN: &str = concat!("</", $element, ">");
        get_value($contents, START_PATTERN, END_PATTERN)
    }};
}

fn parse_release_date(raw: String) -> String {
    if raw.len() >= 8 {
        let year = &raw[0..4];
        let month = &raw[4..6];
        let day = &raw[6..8];

        format!("{year}-{month}-{day}")
    } else {
        raw
    }
}

#[derive(Debug, Clone)]
pub struct HomebrewAppMeta {
    pub name: String,
    pub version: String,
    pub release_date: String,
    pub coder: String,
    pub short_description: String,
    pub long_description: String,
}

impl HomebrewAppMeta {
    pub fn parse(app_path: impl Into<PathBuf>) -> Result<Self, Error> {
        let mut path = app_path.into();
        path.push("meta.xml");
        let contents = fs::read_to_string(&path)?;

        let root = get_property!(&contents, "app").ok_or(Error::InvalidHomebrewAppMeta)?;

        let name = get_property!(&root, "name").ok_or(Error::InvalidHomebrewAppMeta)?;
        let version = get_property!(&root, "version").ok_or(Error::InvalidHomebrewAppMeta)?;
        let release_date =
            get_property!(&root, "release_date").ok_or(Error::InvalidHomebrewAppMeta)?;
        let coder = get_property!(&root, "coder").ok_or(Error::InvalidHomebrewAppMeta)?;
        let short_description =
            get_property!(&root, "short_description").ok_or(Error::InvalidHomebrewAppMeta)?;
        let long_description =
            get_property!(&root, "long_description").ok_or(Error::InvalidHomebrewAppMeta)?;

        let release_date = parse_release_date(release_date);

        Ok(Self {
            name,
            version,
            release_date,
            coder,
            short_description,
            long_description,
        })
    }
}
