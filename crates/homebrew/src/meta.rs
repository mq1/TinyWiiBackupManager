// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::{Context, Result};
use std::{fs, path::Path};

fn get_property<'a>(contents: &'a str, element: &str) -> Option<&'a str> {
    let (_, after_open) = contents
        .match_indices('<')
        .find_map(|(i, _)| contents[i + 1..].strip_prefix(element))?
        .split_once('>')?;

    after_open.match_indices("</").find_map(|(i, _)| {
        after_open[i + 2..]
            .strip_prefix(element)?
            .starts_with('>')
            .then(|| after_open[..i].trim())
    })
}

fn parse_release_date(raw: &str) -> Box<str> {
    if raw.len() >= 8 {
        let mut s = String::with_capacity(10);

        s.push_str(&raw[0..4]);
        s.push('-');
        s.push_str(&raw[4..6]);
        s.push('-');
        s.push_str(&raw[6..8]);

        s.into()
    } else {
        raw.into()
    }
}

#[derive(Debug, Clone)]
pub struct HomebrewAppMeta {
    name: Box<str>,
    version: Box<str>,
    coder: Box<str>,
    short_description: Box<str>,
    long_description: Box<str>,
    release_date: Box<str>,
}

impl HomebrewAppMeta {
    pub fn parse(app_path: &Path) -> Result<Self> {
        let path = app_path.join("meta.xml");
        let raw = fs::read_to_string(&path)?;

        let root = get_property(&raw, "app").context("<app> not found")?;

        let name = get_property(root, "name").context("<name> not found")?;
        let version = get_property(root, "version").context("<version> not found")?;

        let release_date = get_property(root, "release_date").unwrap_or_default();
        let coder = get_property(root, "coder").unwrap_or_default();
        let short_description = get_property(root, "short_description").unwrap_or_default();
        let long_description = get_property(root, "long_description").unwrap_or_default();

        Ok(Self {
            name: name.into(),
            version: version.into(),
            release_date: parse_release_date(release_date),
            coder: coder.into(),
            short_description: short_description.into(),
            long_description: long_description.into(),
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
