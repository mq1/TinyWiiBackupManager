// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    games::{
        banners::download_all_banners,
        covers::{download_all_covers_for_usbloadergx, download_all_covers_for_wiiflow},
    },
    toolbox::{ToolboxGroup, ToolboxItem},
    util::http::download_and_extract_zip,
};
use anyhow::anyhow;
use itertools::Itertools;
use lucide_icons::Icon;

pub const ALL: &[ToolboxGroup] = {
    &[ToolboxGroup {
        label: "Game Backup Loaders",
        icon: Icon::Loader,
        items: &[
            ToolboxItem {
                label: "Download covers for USB Loader GX",
                run_fn: |ctx| {
                    Box::pin(async move {
                        match download_all_covers_for_usbloadergx(ctx.game_ids, &ctx.config).await {
                            errored if !errored.is_empty() => Err(anyhow!(
                                "Covers downloaded successfully, except: {}",
                                errored.iter().format(", ")
                            )),
                            _ => Ok("Covers successfully downloaded".to_string()),
                        }
                    })
                },
            },
            ToolboxItem {
                label: "Download banners for USB Loader GX (GameCube)",
                run_fn: |ctx| {
                    Box::pin(async move {
                        match download_all_banners(ctx.game_ids, &ctx.config.mount_point).await {
                            errored if !errored.is_empty() => Err(anyhow!(
                                "Banners downloaded successfully, except: {}",
                                errored.iter().format(", ")
                            )),
                            _ => Ok("Banners successfully downloaded".to_string()),
                        }
                    })
                },
            },
            ToolboxItem {
                label: "Download wiitdb.xml for USB Loader GX",
                run_fn: |ctx| {
                    Box::pin(async move {
                        download_and_extract_zip(
                            "https://www.gametdb.com/wiitdb.zip",
                            &ctx.config.mount_point.join("apps").join("usbloader_gx"),
                        )
                        .await?;

                        Ok("wiitdb.xml successfully downloaded".to_string())
                    })
                },
            },
            ToolboxItem {
                label: "Download covers for Wiiflow",
                run_fn: |ctx| {
                    Box::pin(async move {
                        match download_all_covers_for_wiiflow(ctx.game_ids, &ctx.config).await {
                            errored if !errored.is_empty() => Err(anyhow!(
                                "Covers downloaded successfully, except: {}",
                                errored.iter().format(", ")
                            )),
                            _ => Ok("Covers successfully downloaded".to_string()),
                        }
                    })
                },
            },
        ],
    }]
};
