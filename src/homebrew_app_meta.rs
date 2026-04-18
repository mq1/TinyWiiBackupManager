// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::HomebrewAppMeta;
use anyhow::{Result, bail};
use slint::{SharedString, ToSharedString};
use std::{fs, path::Path};

impl HomebrewAppMeta {
    pub fn try_from_path(path: &Path) -> Result<Self> {
        let meta = fs::read_to_string(path)?;
        let mut meta = quick_xml::de::from_str::<HomebrewAppMeta>(&meta)?;

        meta.name = parse_name(meta.name);

        if let Ok(date) = parse_date(&meta.release_date) {
            meta.release_date = date;
        }

        Ok(meta)
    }
}

// some apps seem to place " " in front of the name to prioritize themselves when sorting
fn parse_name(raw: SharedString) -> SharedString {
    if raw.chars().next() == Some(' ') {
        raw[1..].to_shared_string()
    } else {
        raw
    }
}

fn parse_date(raw: &str) -> Result<SharedString> {
    if raw.len() < 8 {
        bail!("too short");
    }

    let year = &raw[0..4];
    let month = &raw[4..6];
    let day = &raw[6..8];

    let date = format!("{year}-{month}-{day}").to_shared_string();

    Ok(date)
}
