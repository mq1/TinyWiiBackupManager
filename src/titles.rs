// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{dirs, http::AGENT};
use anyhow::{Result, anyhow};
use std::{fs, sync::OnceLock};

type Titles = Box<[([u8; 6], String)]>;

const DOWNLOAD_URL: &str = "https://www.gametdb.com/wiitdb.txt";

static TITLES_CACHE: OnceLock<Titles> = OnceLock::new();

fn id_to_bytes(id: &str) -> [u8; 6] {
    let mut id_bytes = [0u8; 6];
    let bytes = id.as_bytes();
    let len = bytes.len().min(6);
    id_bytes[..len].copy_from_slice(&bytes[..len]);

    id_bytes
}

pub fn init() -> Result<()> {
    let data_dir = dirs::data_dir()?;

    let path = data_dir.join("titles.txt");
    if !path.exists() {
        let mut res = AGENT.get(DOWNLOAD_URL).call()?;
        let bytes = res.body_mut().read_to_vec()?;
        fs::write(&path, bytes)?;
    }

    let mut titles = Vec::new();
    for line in fs::read_to_string(&path)?.lines() {
        let (id, title) = line.split_once(" = ").unwrap_or_default();
        titles.push((id_to_bytes(id), title.to_string()));
    }

    // We sort it now so we can binary search
    titles.sort_by_key(|(id, _)| *id);

    let titles = titles.into_boxed_slice();
    TITLES_CACHE
        .set(titles)
        .map_err(|_| anyhow!("Failed to set TITLES_CACHE"))?;

    Ok(())
}

pub fn get_title(id: &str) -> Option<String> {
    let id_bytes = id_to_bytes(id);
    let cache = TITLES_CACHE.get()?;
    let index = cache.binary_search_by_key(&id_bytes, |(id, _)| *id).ok()?;
    let title = cache[index].1.clone();

    Some(title)
}
