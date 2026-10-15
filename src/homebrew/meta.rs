// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::{Context, Result};
use self_cell::self_cell;
use std::{borrow::Cow, fs, path::Path};

self_cell!(
    pub struct OwnedHomebrewAppMeta {
        owner: String,

        #[covariant]
        dependent: HomebrewAppMetaContent,
    }

    impl {Debug}
);

fn get_value<'a>(contents: &'a str, start_pattern: &str, end_pattern: &str) -> Option<&'a str> {
    let start = contents.find(start_pattern)?;
    let slice = &contents[start..];
    let start = slice.find('>')? + 1;
    let slice = &slice[start..];
    let end = slice.find(end_pattern)?;
    let value = &slice[..end];

    Some(value.trim())
}

macro_rules! get_property {
    ($contents:expr, $element:literal) => {{
        const START_PATTERN: &str = concat!("<", $element);
        const END_PATTERN: &str = concat!("</", $element, ">");
        const ERROR: &str = concat!("failed to parse meta.xml: ", $element, " not found");

        get_value($contents, START_PATTERN, END_PATTERN).context(ERROR)
    }};
}

fn parse_release_date<'a>(raw: &'a str) -> Cow<'a, str> {
    if raw.len() >= 8 {
        let mut s = String::with_capacity(10);

        s.push_str(&raw[0..4]);
        s.push('-');
        s.push_str(&raw[4..6]);
        s.push('-');
        s.push_str(&raw[6..8]);

        Cow::Owned(s)
    } else {
        Cow::Borrowed(raw)
    }
}

#[derive(Debug, Clone)]
pub struct HomebrewAppMetaContent<'a> {
    name: &'a str,
    version: &'a str,
    coder: &'a str,
    short_description: &'a str,
    long_description: &'a str,
    release_date: Cow<'a, str>,
}

#[derive(Debug)]
pub struct HomebrewAppMeta(OwnedHomebrewAppMeta);

impl Clone for HomebrewAppMeta {
    fn clone(&self) -> Self {
        let raw = self.0.borrow_owner().clone();
        Self(OwnedHomebrewAppMeta::from_raw(raw).unwrap())
    }
}

impl OwnedHomebrewAppMeta {
    fn from_raw(raw: String) -> Result<Self> {
        Self::try_new(raw, |raw| {
            let root = get_property!(&raw, "app")?;

            let name = get_property!(&root, "name")?;
            let version = get_property!(&root, "version")?;

            let release_date = get_property!(&root, "release_date").unwrap_or_default();
            let coder = get_property!(&root, "coder").unwrap_or_default();
            let short_description = get_property!(&root, "short_description").unwrap_or_default();
            let long_description = get_property!(&root, "long_description").unwrap_or_default();

            let release_date = parse_release_date(release_date);

            Ok(HomebrewAppMetaContent {
                name,
                version,
                release_date,
                coder,
                short_description,
                long_description,
            })
        })
    }
}

impl HomebrewAppMeta {
    pub fn parse(app_path: &Path) -> Result<Self> {
        let path = app_path.join("meta.xml");
        let raw = fs::read_to_string(&path)?;
        OwnedHomebrewAppMeta::from_raw(raw).map(HomebrewAppMeta)
    }

    pub fn name(&self) -> &str {
        self.0.borrow_dependent().name
    }

    pub fn version(&self) -> &str {
        self.0.borrow_dependent().version
    }

    pub fn release_date(&self) -> &str {
        &self.0.borrow_dependent().release_date
    }

    pub fn coder(&self) -> &str {
        self.0.borrow_dependent().coder
    }

    pub fn short_description(&self) -> &str {
        self.0.borrow_dependent().short_description
    }

    pub fn long_description(&self) -> &str {
        self.0.borrow_dependent().long_description
    }
}
