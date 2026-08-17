// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::SortBy, errors::Error, homebrew::homebrew_app::HomebrewApp, util::misc::unzip,
};
use smol::{fs, stream::StreamExt};
use std::path::Path;

pub mod homebrew_app;
pub mod homebrew_app_list;
pub mod meta;

async fn scan_dir(dir_path: &Path) -> Vec<HomebrewApp> {
    let Ok(entries) = fs::read_dir(dir_path).await else {
        return Vec::new();
    };

    entries
        .then(async |entry| {
            let path = entry?.path();
            HomebrewApp::try_from_path(path).await
        })
        .filter_map(Result::ok)
        .collect::<Vec<_>>()
        .await
}

pub async fn list(root_path: &Path, sort_by: SortBy) -> Result<Vec<HomebrewApp>, Error> {
    let apps_dir_path = root_path.join("apps");
    let mut apps = scan_dir(&apps_dir_path).await;
    sort(&mut apps, sort_by);

    Ok(apps)
}

pub fn sort(apps: &mut [HomebrewApp], sort_by: SortBy) {
    apps.sort_unstable_by(|a, b| match sort_by {
        SortBy::NameDescending => a.meta.name.cmp(&b.meta.name),
        SortBy::NameAscending => b.meta.name.cmp(&a.meta.name),
        SortBy::SizeDescending => a.size.cmp(&b.size),
        SortBy::SizeAscending => b.size.cmp(&a.size),
    });
}

pub async fn import(
    root_path: impl AsRef<Path>,
    paths: impl IntoIterator<Item = impl AsRef<Path>>,
) -> Result<usize, Error> {
    let mut count = 0;

    for path in paths {
        unzip(path, root_path.as_ref()).await?;
        count += 1;
    }

    Ok(count)
}
