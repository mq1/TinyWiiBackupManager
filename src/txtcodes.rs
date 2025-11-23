// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::app::App;
use crate::messages::Message;
use crate::{games::GameID, http};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TxtCodesSource {
    WebArchive,
    Rc24,
}

impl TxtCodesSource {
    fn get_base_url(&self) -> &str {
        match self {
            Self::WebArchive => "https://web.archive.org/web/202009if_/geckocodes.org/",
            Self::Rc24 => "https://codes.rc24.xyz/",
        }
    }

    pub fn get_comment(&self) -> &str {
        match self {
            Self::WebArchive => {
                "https://web.archive.org/web/202009if_/geckocodes.org/    (Recommended, high quality)"
            }
            Self::Rc24 => "https://codes.rc24.xyz/",
        }
    }

    pub fn get_url(&self, game_id: [u8; 6]) -> String {
        format!("{}txt.php?txt={}", self.get_base_url(), game_id.as_str())
    }
}

fn download_cheats_for_game(
    txt_cheatcodespath: &Path,
    source: TxtCodesSource,
    game_id: [u8; 6],
) -> Result<()> {
    let path = txt_cheatcodespath
        .join(game_id.as_str())
        .with_extension("txt");

    if path.exists() {
        return Ok(());
    }

    let url = source.get_url(game_id);
    http::download_file(&url, &path)?;

    Ok(())
}

pub fn spawn_download_cheats_task(app: &App) {
    let txt_cheatcodespath = app.config.contents.mount_point.join("txtcodes");
    let source = app.config.contents.txt_codes_source;
    let games = app.games.clone().into_boxed_slice();

    app.task_processor.spawn(move |msg_sender| {
        msg_sender.send(Message::UpdateStatus(
            "📓 Downloading cheats...".to_string(),
        ))?;

        fs::create_dir_all(&txt_cheatcodespath)?;

        for game in &games {
            msg_sender.send(Message::UpdateStatus(format!(
                "📓 Downloading cheats... ({})",
                &game.display_title
            )))?;

            if let Err(e) = download_cheats_for_game(&txt_cheatcodespath, source, game.id) {
                let context = format!("Failed to download cheats for {}", &game.display_title);
                msg_sender.send(Message::NotifyError(e.context(context)))?;
            }
        }

        msg_sender.send(Message::NotifyInfo("📓 Cheats downloaded".to_string()))?;

        Ok(())
    });
}
