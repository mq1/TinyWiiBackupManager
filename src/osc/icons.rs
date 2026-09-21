// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::util::http::download_file;
use smol::{
    fs,
    stream::{self, Stream, StreamExt},
};
use smol_str::SmolStr;
use std::{convert::identity, path::PathBuf};

pub fn download_all_icons(
    slugs_and_icon_uris: impl IntoIterator<Item = (SmolStr, SmolStr)>,
    data_dir: PathBuf,
) -> impl Stream<Item = SmolStr> {
    let icons_dir = data_dir.join("osc-icons");

    let it = slugs_and_icon_uris.into_iter().map(move |(slug, uri)| {
        let path = icons_dir.join(&slug).with_added_extension("png");
        (slug, uri, path)
    });

    stream::iter(it)
        .then(move |(slug, uri, path)| async move {
            if fs::metadata(&path).await.is_ok_and(|meta| meta.is_file()) {
                None
            } else {
                download_file(&uri, &path).await.ok().map(|_| slug)
            }
        })
        .filter_map(identity)
}
