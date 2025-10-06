// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::http::AGENT;
use anyhow::Result;
use std::{fs, path::Path};

const DOWNLOAD_URL: &str = "https://www.gametdb.com/wiitdb.txt";

pub struct Titles(Box<[([u8; 6], String)]>);

impl Titles {
    pub fn load(data_dir: &Path) -> Result<Self> {
        let path = data_dir.join("titles.txt");
        if !path.exists() {
            let mut res = AGENT.get(DOWNLOAD_URL).call()?;
            let bytes = res.body_mut().read_to_vec()?;
            fs::write(&path, bytes)?;
        }

        let bytes = fs::read(&path).unwrap_or_default();

        let mut titles = Vec::new();
        for line in String::from_utf8_lossy(&bytes).lines() {
            let (id, title) = line.split_once(" = ").unwrap_or_default();
            titles.push((id_to_bytes(id), title.to_string()));
        }

        // We sort it now so we can binary search
        titles.sort_by_key(|(id, _)| *id);

        Ok(Titles(titles.into_boxed_slice()))
    }

    pub fn get(&self, id: &str) -> Option<String> {
        let id_bytes = id_to_bytes(id);
        let index = self.0.binary_search_by_key(&id_bytes, |(id, _)| *id).ok()?;
        let title = self.0[index].1.clone();

        Some(title)
    }
}

fn id_to_bytes(id: &str) -> [u8; 6] {
    let mut id_bytes = [0u8; 6];
    let bytes = id.as_bytes();
    let len = bytes.len().min(6);
    id_bytes[..len].copy_from_slice(&bytes[..len]);

    id_bytes
}
