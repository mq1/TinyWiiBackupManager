// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::{Context, Result};
use std::{fs, path::Path};

fn get_value(contents: &str, start_pattern: &str, end_pattern: &str) -> Option<Box<str>> {
    let start = contents.find(start_pattern)?;
    let slice = &contents[start..];
    let start = slice.find('>')? + 1;
    let slice = &slice[start..];
    let end = slice.find(end_pattern)?;
    let value = &slice[..end];

    Some(value.trim().into())
}

macro_rules! get_property {
    ($contents:expr, $element:literal) => {{
        const START_PATTERN: &str = concat!("<", $element);
        const END_PATTERN: &str = concat!("</", $element, ">");
        const ERROR: &str = concat!("failed to parse meta.xml: ", $element, " not found");

        get_value($contents, START_PATTERN, END_PATTERN).context(ERROR)
    }};
}

fn parse_release_date(raw: Box<str>) -> Box<str> {
    if raw.len() >= 8 {
        let mut s = String::with_capacity(10);

        s.push_str(&raw[0..4]);
        s.push('-');
        s.push_str(&raw[4..6]);
        s.push('-');
        s.push_str(&raw[6..8]);

        s.into_boxed_str()
    } else {
        raw
    }
}

#[derive(Debug, Clone)]
pub struct HomebrewAppMeta {
    name: Box<str>,
    version: Box<str>,
    release_date: Box<str>,
    coder: Box<str>,
    short_description: Box<str>,
    long_description: Box<str>,
}

impl HomebrewAppMeta {
    pub fn parse(app_path: &Path) -> Result<Self> {
        let path = app_path.join("meta.xml");
        let contents = fs::read_to_string(&path)?;

        let root = get_property!(&contents, "app")?;

        let name = get_property!(&root, "name")?;
        let version = get_property!(&root, "version")?;

        let release_date = get_property!(&root, "release_date").unwrap_or_default();
        let coder = get_property!(&root, "coder").unwrap_or_default();
        let short_description = get_property!(&root, "short_description").unwrap_or_default();
        let long_description = get_property!(&root, "long_description").unwrap_or_default();

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
