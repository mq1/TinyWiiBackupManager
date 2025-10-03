// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use crate::{PROJ, http::AGENT, show_err};
use std::{fs, sync::LazyLock};

type Index = Box<[([u8; 6], String)]>;

const DOWNLOAD_URL: &str = "https://www.gametdb.com/wiitdb.txt";

fn id_to_bytes(id: &str) -> [u8; 6] {
    let mut id_bytes = [0u8; 6];
    let bytes = id.as_bytes();
    let len = bytes.len().min(6);
    id_bytes[..len].copy_from_slice(&bytes[..len]);

    id_bytes
}

static TITLES_CACHE: LazyLock<Index> = LazyLock::new(|| {
    let res = || -> Result<Index> {
        let data_dir = PROJ.data_dir();
        let _ = fs::create_dir_all(data_dir);

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

        Ok(titles.into_boxed_slice())
    }();

    match res {
        Ok(slice) => slice,
        Err(e) => {
            show_err(&e);
            Box::new([])
        }
    }
});

pub fn get_title(id: &str) -> Option<String> {
    let id_bytes = id_to_bytes(id);
    let index = TITLES_CACHE
        .binary_search_by_key(&id_bytes, |(id, _)| *id)
        .ok()?;

    Some(TITLES_CACHE[index].1.clone())
}
