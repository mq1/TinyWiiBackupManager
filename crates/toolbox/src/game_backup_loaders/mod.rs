// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

mod banners;
pub mod covers;

use crate::{
    ToolboxGroup, ToolboxItem,
    game_backup_loaders::{
        banners::download_all_banners,
        covers::{download_all_covers_for_usbloadergx, download_all_covers_for_wiiflow},
    },
};
use anyhow::{Context, anyhow};
use itertools::Itertools;
use twbm_http::download_and_extract_zip;

pub const ALL: &[ToolboxGroup] = {
    &[ToolboxGroup {
        label: "Game Backup Loaders",
        icon: "loader",
        items: &[
            ToolboxItem {
                label: "Download covers for USB Loader GX",
                run: |ctx| match download_all_covers_for_usbloadergx(ctx.game_ids, &ctx.config) {
                    errored if !errored.is_empty() => Err(anyhow!(
                        "Finished downloading covers. Failed: {}",
                        errored.iter().format(", ")
                    )),
                    _ => Ok("Covers successfully downloaded".to_string()),
                },
            },
            ToolboxItem {
                label: "Download banners for USB Loader GX (GameCube)",
                run: |ctx| {
                    let mount_point = ctx
                        .config
                        .mount_point
                        .as_deref()
                        .context("Invalid mount point")?;

                    match download_all_banners(ctx.game_ids, mount_point) {
                        errored if !errored.is_empty() => Err(anyhow!(
                            "Finished downloading banners: Failed: {}",
                            errored.iter().format(", ")
                        )),
                        _ => Ok("Banners successfully downloaded".to_string()),
                    }
                },
            },
            ToolboxItem {
                label: "Download wiitdb.xml for USB Loader GX",
                run: |ctx| {
                    let mount_point = ctx
                        .config
                        .mount_point
                        .as_deref()
                        .context("Invalid mount point")?;

                    let dest = mount_point.join("apps").join("usbloader_gx");

                    download_and_extract_zip("https://www.gametdb.com/wiitdb.zip", &dest)?;

                    Ok("wiitdb.xml successfully downloaded".to_string())
                },
            },
            ToolboxItem {
                label: "Download covers for Wiiflow",
                run: |ctx| match download_all_covers_for_wiiflow(ctx.game_ids, &ctx.config) {
                    errored if !errored.is_empty() => Err(anyhow!(
                        "Finished downloading covers. Failed: {}",
                        errored.iter().format(", ")
                    )),
                    _ => Ok("Covers successfully downloaded".to_string()),
                },
            },
        ],
    }]
};
