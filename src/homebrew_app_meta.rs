// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::HomebrewAppMeta;
use anyhow::Result;
use slint::{SharedString, ToSharedString};
use std::{fs, path::Path};

impl HomebrewAppMeta {
    pub fn try_from_path(path: &Path) -> Result<Self> {
        let meta = fs::read_to_string(path)?;
        let mut meta = quick_xml::de::from_str::<HomebrewAppMeta>(&meta)?;

        meta.name = parse_name(meta.name);
        meta.release_date = parse_date(meta.release_date);

        Ok(meta)
    }
}

// some apps seem to place " " in front of the name to prioritize themselves when sorting
fn parse_name(raw: SharedString) -> SharedString {
    if raw.starts_with(' ') {
        raw[1..].to_shared_string()
    } else {
        raw
    }
}

fn parse_date(raw: SharedString) -> SharedString {
    if raw.len() >= 8 {
        let year = &raw[0..4];
        let month = &raw[4..6];
        let day = &raw[6..8];

        format!("{year}-{month}-{day}").to_shared_string()
    } else {
        raw
    }
}
