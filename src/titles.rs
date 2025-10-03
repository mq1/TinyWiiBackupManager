// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use crate::{PROJ, http::AGENT};
use std::fs;

const DOWNLOAD_URL: &str = "https://www.gametdb.com/wiitdb.txt";

fn id_to_bytes(id: &str) -> [u8; 6] {
    let mut id_bytes = [0u8; 6];
    let bytes = id.as_bytes();
    let len = bytes.len().min(6);
    id_bytes[..len].copy_from_slice(&bytes[..len]);

    id_bytes
}

pub struct Titles(Box<[([u8; 6], String)]>);

impl Titles {
    pub fn get() -> Result<Titles> {
        let data_dir = PROJ.data_dir();
        fs::create_dir_all(data_dir)?;

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

        let index = titles.into_boxed_slice();
        Ok(Titles(index))
    }

    pub fn get_title(&self, id: &str) -> Option<String> {
        let id_bytes = id_to_bytes(id);
        let index = self.0.binary_search_by_key(&id_bytes, |(id, _)| *id).ok()?;
        Some(self.0[index].1.clone())
    }
}
