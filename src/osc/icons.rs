// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{osc::osc_state::OscAppList, util::http::download_file};

use smol::{
    fs,
    stream::{self, Stream, StreamExt},
};
use std::{convert::identity, path::Path};

pub fn download_all_icons(apps: OscAppList, data_dir: &'static Path) -> impl Stream<Item = usize> {
    let icons_dir = data_dir.join("osc-icons");

    stream::iter(apps.into_iter().enumerate())
        .then(move |(idx, app)| {
            let icons_dir = icons_dir.clone();

            async move {
                let path = icons_dir.join(app.slug()).with_added_extension("png");

                if fs::metadata(&path).await.is_ok_and(|meta| meta.is_file()) {
                    None
                } else {
                    download_file(app.icon_uri(), &path).await.ok().map(|_| idx)
                }
            }
        })
        .filter_map(identity)
}
