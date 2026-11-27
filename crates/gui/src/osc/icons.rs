// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use smol::{channel::unbounded, stream::Stream};
use std::path::Path;
use twbm_http::download_file;
use twbm_osc::manager::OscAppList;

pub fn download_all_icons(apps: OscAppList, data_dir: &Path) -> impl Stream<Item = Box<str>> {
    let icons_dir = data_dir.join("osc-icons");

    let (tx, rx) = unbounded();

    std::thread::spawn(move || {
        for app in apps.iter() {
            let path = icons_dir.join(format!("{}.png", app.slug()));
            if matches!(download_file(app.icon_uri(), &path), Ok(true)) {
                let _ = tx.try_send(app.slug().into());
            }
        }
    });

    rx
}
