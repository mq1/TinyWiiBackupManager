// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;
use jiff::civil::Date;
use serde::{Deserialize, Deserializer};
use serde_roxmltree::Options;
use std::{fs, path::Path};

#[derive(Debug, Clone, Deserialize)]
pub struct HomebrewAppMeta {
    name: Box<str>,
    version: Box<str>,

    #[serde(default)]
    coder: Box<str>,
    #[serde(default)]
    short_description: Box<str>,
    #[serde(default)]
    long_description: Box<str>,
    #[serde(default, deserialize_with = "parse_release_date")]
    release_date: Date,
}

impl HomebrewAppMeta {
    pub fn parse(app_path: &Path) -> Result<Self> {
        let path = app_path.join("meta.xml");
        let raw = fs::read_to_string(&path)?;
        let meta = serde_roxmltree::defaults().prefix_attr().from_str(&raw)?;

        Ok(meta)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn release_date(&self) -> Date {
        self.release_date
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

fn parse_release_date<'de, D>(deserializer: D) -> Result<Date, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    let date = s
        .trim()
        .get(..8)
        .and_then(|date_raw| date_raw.parse().ok())
        .unwrap_or_default();

    Ok(date)
}
