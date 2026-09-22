// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::{Context, Result};
use compact_str::{CompactString, ToCompactString, format_compact};
use std::{fs, path::Path};

fn get_value(contents: &str, start_pattern: &str, end_pattern: &str) -> Option<CompactString> {
    let start = contents.find(start_pattern)?;
    let slice = &contents[start..];
    let start = slice.find('>')? + 1;
    let slice = &slice[start..];
    let end = slice.find(end_pattern)?;
    let value = &slice[..end];

    Some(value.trim().to_compact_string())
}

macro_rules! get_property {
    ($contents:expr, $element:literal) => {{
        const START_PATTERN: &str = concat!("<", $element);
        const END_PATTERN: &str = concat!("</", $element, ">");
        get_value($contents, START_PATTERN, END_PATTERN).context("failed to parse meta.xml")
    }};
}

fn parse_release_date(raw: CompactString) -> CompactString {
    if raw.len() >= 8 {
        let year = &raw[0..4];
        let month = &raw[4..6];
        let day = &raw[6..8];

        format_compact!("{year}-{month}-{day}")
    } else {
        raw
    }
}

#[derive(Debug, Clone)]
pub struct HomebrewAppMeta {
    name: CompactString,
    version: CompactString,
    release_date: CompactString,
    coder: CompactString,
    short_description: CompactString,
    long_description: CompactString,
}

impl HomebrewAppMeta {
    pub fn parse(app_path: &Path) -> Result<Self> {
        let path = app_path.join("meta.xml");
        let contents = fs::read_to_string(&path)?;

        let root = get_property!(&contents, "app")?;

        let name = get_property!(&root, "name")?;
        let version = get_property!(&root, "version")?;
        let release_date = get_property!(&root, "release_date")?;
        let coder = get_property!(&root, "coder")?;
        let short_description = get_property!(&root, "short_description")?;
        let long_description = get_property!(&root, "long_description")?;

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

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn release_date(&self) -> &str {
        &self.release_date
    }

    pub fn coder(&self) -> &str {
        &self.coder
    }

    pub fn short_description(&self) -> &str {
        &self.short_description
    }

    pub fn long_description(&self) -> &str {
        &self.long_description
    }
}
