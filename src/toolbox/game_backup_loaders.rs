// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    errors::Error,
    games::covers::{download_all_covers_for_usbloadergx, download_all_covers_for_wiiflow},
    toolbox::{ToolContext, ToolboxGroup, ToolboxItem},
};
use async_trait::async_trait;
use lucide_icons::Icon;

#[derive(Debug)]
pub struct DownloadCoversForUSBLoaderGX;

#[async_trait]
impl ToolboxItem for DownloadCoversForUSBLoaderGX {
    fn label(&self) -> &'static str {
        "Download covers for USB Loader GX"
    }

    async fn run(&self, ctx: ToolContext) -> Result<String, Error> {
        match download_all_covers_for_usbloadergx(ctx.game_ids, &ctx.config).await {
            errored if !errored.is_empty() => Err(Error::DownloadCovers(errored)),
            _ => Ok("Covers successfully downloaded".to_string()),
        }
    }
}

#[derive(Debug)]
pub struct DownloadCoversForWiiflow;

#[async_trait]
impl ToolboxItem for DownloadCoversForWiiflow {
    fn label(&self) -> &'static str {
        "Download covers for Wiiflow"
    }

    async fn run(&self, ctx: ToolContext) -> Result<String, Error> {
        match download_all_covers_for_wiiflow(ctx.game_ids, &ctx.config).await {
            errored if !errored.is_empty() => Err(Error::DownloadCovers(errored)),
            _ => Ok("Covers successfully downloaded".to_string()),
        }
    }
}

pub const ALL: &[ToolboxGroup] = {
    &[ToolboxGroup {
        label: "Game Backup Loaders",
        icon: Icon::Loader,
        items: &[&DownloadCoversForUSBLoaderGX, &DownloadCoversForWiiflow],
    }]
};
