// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{HbcApp, HbcAppMeta, SortBy, util::MIB};
use slint::ToSharedString;
use std::{fs, path::Path};

impl HbcApp {
    pub fn maybe_from_path(path: &Path) -> Option<Self> {
        if !path.is_dir() {
            return None;
        }

        let filename = path.file_name()?.to_str()?;
        if filename.starts_with('.') {
            return None;
        }

        let meta_path = path.join("meta.xml");
        let meta = fs::read_to_string(&meta_path).ok()?;
        let meta = quick_xml::de::from_str::<HbcAppMeta>(&meta).ok()?;

        let size = fs_extra::dir::get_size(path).unwrap_or(0);

        #[allow(clippy::cast_precision_loss)]
        let size_mib = size as f32 / MIB;

        let app = Self {
            path: path.to_string_lossy().to_shared_string(),
            meta,
            size_mib,
        };

        Some(app)
    }
}

pub fn get_compare_fn(sort_by: SortBy) -> fn(&HbcApp, &HbcApp) -> std::cmp::Ordering {
    match sort_by {
        SortBy::NameAscending => |a, b| a.meta.name.cmp(&b.meta.name),
        SortBy::NameDescending => |a, b| b.meta.name.cmp(&a.meta.name),
        SortBy::SizeAscending => |a, b| a.size_mib.total_cmp(&b.size_mib),
        SortBy::SizeDescending => |a, b| b.size_mib.total_cmp(&a.size_mib),
    }
}
