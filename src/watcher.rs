// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    MainWindow, refresh_disk_usage, refresh_games, refresh_hbc_apps, show_err, titles::Titles,
};
use anyhow::Result;
use notify::{
    Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
    event::{CreateKind, ModifyKind, RemoveKind, RenameMode},
};
use slint::Weak;
use std::{path::Path, sync::Arc};

pub fn init_watcher(
    weak: Weak<MainWindow>,
    mount_point: &Path,
    titles: &Arc<Titles>,
) -> Result<Option<RecommendedWatcher>> {
    if mount_point.as_os_str().is_empty() {
        return Ok(None);
    }

    let path = mount_point.to_path_buf();
    let titles = titles.clone();

    let mut watcher = notify::recommended_watcher(move |res: Result<Event, _>| {
        if let Ok(event) = res
            && matches!(
                event.kind,
                EventKind::Create(CreateKind::Folder)
                    | EventKind::Modify(ModifyKind::Name(RenameMode::Any))
                    | EventKind::Remove(RemoveKind::Folder)
            )
        {
            let path = path.clone();
            let titles = titles.clone();

            let _ = weak.upgrade_in_event_loop(move |handle| {
                if let Err(e) = refresh_games(&handle, &path, &titles) {
                    show_err(e);
                }
                if let Err(e) = refresh_hbc_apps(&handle, &path) {
                    show_err(e);
                }
                refresh_disk_usage(&handle, &path);
            });
        }
    })?;

    watcher.watch(mount_point, RecursiveMode::Recursive)?;

    Ok(Some(watcher))
}
