// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    errors::Error,
    games::covers::{download_all_covers_for_usbloadergx, download_all_covers_for_wiiflow},
    toolbox::{ToolboxGroup, ToolboxItem},
};
use iced::Task;
use lucide_icons::Icon;

pub const ALL: &[ToolboxGroup] = {
    &[ToolboxGroup {
        label: "Game Backup Loaders",
        icon: Icon::Loader,
        items: &[
            ToolboxItem {
                label: "Download covers for USB Loader GX",
                run: |ctx| {
                    Task::future(async move {
                        match download_all_covers_for_usbloadergx(ctx.game_ids, &ctx.config).await {
                            errored if !errored.is_empty() => Err(Error::DownloadCovers(errored)),
                            _ => Ok("Covers successfully downloaded".to_string()),
                        }
                    })
                },
            },
            ToolboxItem {
                label: "Download covers for Wiiflow",
                run: |ctx| {
                    Task::future(async move {
                        match download_all_covers_for_wiiflow(ctx.game_ids, &ctx.config).await {
                            errored if !errored.is_empty() => Err(Error::DownloadCovers(errored)),
                            _ => Ok("Covers successfully downloaded".to_string()),
                        }
                    })
                },
            },
        ],
    }]
};
