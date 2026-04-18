// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{Game, SortBy, data_dir::DATA_DIR, id_map, util::GIB};
use slint::{Image, ToSharedString};
use std::{cmp::Ordering, path::Path};

impl Game {
    #[must_use]
    pub fn maybe_from_path(path: &Path, is_wii: bool) -> Option<Self> {
        if !path.is_dir() {
            return None;
        }

        let filename = path.file_name()?.to_str()?;
        if filename.starts_with('.') {
            return None;
        }

        let (title_str, id_str) = filename.split_once('[')?;
        let id = id_str.strip_suffix(']')?;
        if !matches!(id.len(), 4 | 6) {
            return None;
        }

        let title = match id_map::get(id) {
            Some(e) => e.title.to_shared_string(),
            None => title_str.trim().to_shared_string(),
        };

        let size = fs_extra::dir::get_size(path).unwrap_or(0);

        #[allow(clippy::cast_precision_loss)]
        let size_gib = size as f32 / GIB;

        let cover_path = DATA_DIR.join("covers").join(format!("{id}.png"));
        let cover = Image::load_from_path(&cover_path).unwrap_or_default();

        Some(Self {
            path: path.to_string_lossy().to_shared_string(),
            is_wii,
            size_gib,
            title,
            id: id.to_shared_string(),
            cover,
        })
    }

    pub fn reload_cover(&mut self) {
        let cover_path = DATA_DIR.join("covers").join(format!("{}.png", self.id));
        let cover = Image::load_from_path(&cover_path).unwrap_or_default();
        self.cover = cover;
    }
}

pub fn get_compare_fn(sort_by: SortBy) -> Box<dyn Fn(&Game, &Game) -> Ordering> {
    match sort_by {
        SortBy::NameAscending => Box::new(|a, b| a.title.cmp(&b.title)),
        SortBy::NameDescending => Box::new(|a, b| b.title.cmp(&a.title)),
        SortBy::SizeAscending => Box::new(|a, b| a.size_gib.total_cmp(&b.size_gib)),
        SortBy::SizeDescending => Box::new(|a, b| b.size_gib.total_cmp(&a.size_gib)),
    }
}
