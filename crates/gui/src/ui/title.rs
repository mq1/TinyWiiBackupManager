// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{state::AppState, ui::pages::Page};
use std::fmt::Write;
use twbm_driveinfo::DriveInfoState;
use twbm_games::manager::GameManager;
use twbm_homebrew::homebrew_app_manager::HomebrewAppManager;

pub fn title(state: &AppState) -> String {
    let mut s = String::with_capacity(128);

    s.push_str(concat!(env!("CARGO_PKG_NAME"), "  ›  "));

    let Some(mount_point) = state.config.contents.mount_point.as_deref() else {
        s.push_str("No drive selected");
        return s;
    };

    let label = mount_point.file_name().unwrap_or(mount_point.as_os_str());
    write!(&mut s, "{}", label.display()).unwrap();

    if let DriveInfoState::Loaded(info) = &state.drive {
        write!(&mut s, "  ({}/{})", info.used_size, info.total_size).unwrap();
    }

    s.push_str("  ›  ");
    s.push_str(state.current_page.as_str());

    match state.current_page {
        Page::Games if let GameManager::Loaded(games) = &state.game_manager => {
            write!(&mut s, "  (x{}", games.count()).unwrap();
            if let DriveInfoState::Loaded(info) = &state.drive {
                write!(&mut s, "  ~{}", info.games_size).unwrap();
            }
            s.push(')');
        }
        Page::HomebrewApps if let HomebrewAppManager::Loaded(apps) = &state.homebrew => {
            write!(&mut s, "  (x{}", apps.count()).unwrap();
            if let DriveInfoState::Loaded(info) = &state.drive {
                write!(&mut s, "  ~{}", info.apps_size).unwrap();
            }
            s.push(')');
        }
        _ => {}
    }

    s
}
