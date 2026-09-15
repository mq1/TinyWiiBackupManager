// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{errors::Error, osc::osc_app::OscApp, util::http::download_file};
use smol::fs;
use std::path::Path;

const CONTENTS_URL: &str = "https://hbb1.oscwii.org/api/v4/contents";

#[derive(Default, Clone, Debug)]
pub enum OscContents {
    #[default]
    NotYetLoaded,
    Loaded(Vec<OscApp>),
    Errored(Error),
}

impl OscContents {
    pub async fn load(data_dir: &Path) -> Self {
        let cache_path = data_dir.join("osc-apps.json");

        let res = async move {
            if !fs::metadata(&cache_path)
                .await
                .is_ok_and(|meta| meta.is_file())
            {
                download_file(CONTENTS_URL, &cache_path).await?;
            }

            let contents = fs::read_to_string(&cache_path).await?;
            let apps = serde_json::from_str(&contents)?;

            Ok(apps)
        }
        .await;

        match res {
            Ok(apps) => Self::Loaded(apps),
            Err(err) => Self::Errored(err),
        }
    }
}
