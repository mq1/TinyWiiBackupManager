// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{MainWindow, config, refresh_disk_usage, refresh_games, refresh_hbc_apps, show_err};
use anyhow::{Result, anyhow};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use slint::ComponentHandle;
use std::sync::Mutex;

static WATCHER: Mutex<Option<RecommendedWatcher>> = Mutex::new(None);

pub fn init(handle: &MainWindow) -> Result<()> {
    let mount_point = config::get().mount_point;
    if mount_point.as_os_str().is_empty() {
        return Ok(());
    }

    let weak = handle.as_weak();
    let mut watcher = notify::recommended_watcher(move |res| {
        if let Ok(notify::Event {
            kind:
                notify::EventKind::Modify(_)
                | notify::EventKind::Create(_)
                | notify::EventKind::Remove(_),
            ..
        }) = res
        {
            weak.upgrade_in_event_loop(|handle| {
                refresh_games(&handle).err().map(show_err);
                refresh_hbc_apps(&handle).err().map(show_err);
                refresh_disk_usage(&handle);
            })
            .err()
            .map(show_err);
        }
    })?;

    watcher.watch(&mount_point.join("wbfs"), RecursiveMode::NonRecursive)?;
    watcher.watch(&mount_point.join("games"), RecursiveMode::NonRecursive)?;
    watcher.watch(&mount_point.join("apps"), RecursiveMode::NonRecursive)?;

    WATCHER
        .lock()
        .map_err(|_| anyhow!("Mutex poisoned"))?
        .replace(watcher);

    Ok(())
}
