// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::util::http::download_file;

use smol::{
    fs,
    stream::{self, Stream, StreamExt},
};
use std::{convert::identity, path::Path};

pub fn download_all_icons(
    slugs_and_icon_uris: impl IntoIterator<Item = (String, String)>,
    data_dir: &'static Path,
) -> impl Stream<Item = usize> {
    let icons_dir = data_dir.join("osc-icons");

    let it = slugs_and_icon_uris
        .into_iter()
        .enumerate()
        .map(move |(idx, (slug, uri))| {
            let path = icons_dir.join(&slug).with_added_extension("png");
            (idx, uri, path)
        });

    stream::iter(it)
        .then(move |(idx, uri, path)| async move {
            if fs::metadata(&path).await.is_ok_and(|meta| meta.is_file()) {
                None
            } else {
                download_file(&uri, &path).await.ok().map(|_| idx)
            }
        })
        .filter_map(identity)
}
