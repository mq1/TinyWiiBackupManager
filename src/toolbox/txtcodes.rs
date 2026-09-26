// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    games::txtcodes::download_cheats,
    toolbox::{ToolboxGroup, ToolboxItem},
};
use itertools::Itertools;
use lucide_icons::Icon;
use wii_disc_info::game_id::GameID;

pub const ALL: &[ToolboxGroup] = {
    &[ToolboxGroup {
        label: "Cheats",
        icon: Icon::Skull,
        items: &[ToolboxItem {
            label: "Download cheats for all games",
            run_fn: |ctx| {
                Box::pin(async move {
                    let mut failed = Vec::with_capacity(ctx.game_ids.len());

                    for game_id in ctx.game_ids {
                        if download_cheats(game_id, &ctx.config).await.is_err() {
                            failed.push(game_id);
                        }
                    }

                    let msg = if failed.is_empty() {
                        "Downloaded cheats for all games".to_string()
                    } else {
                        format!(
                            "Downloaded cheats for all games except for: {}",
                            failed.iter().map(GameID::as_str).format(", ")
                        )
                    };

                    Ok(msg)
                })
            },
        }],
    }]
};
