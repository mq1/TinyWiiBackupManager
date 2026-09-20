// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    games::games_state::GamesState, homebrew::homebrew_state::HomebrewState, state::AppState,
    ui::pages::Page, util::drive_state::DriveState,
};
use std::fmt::Write;

pub fn title(state: &AppState) -> String {
    let mut s = String::with_capacity(128);

    s.push_str(env!("CARGO_PKG_NAME"));
    s.push_str("  ›  ");

    let mount_point = &state.config.mount_point;
    if mount_point.as_os_str().is_empty() {
        s.push_str("No drive selected");
        return s;
    }

    let label = mount_point.file_name().unwrap_or(mount_point.as_os_str());
    write!(&mut s, "{}", label.display()).unwrap();

    if let DriveState::Loaded(info) = &state.drive {
        write!(
            &mut s,
            "  ({}/{})",
            info.used_size_str(),
            info.total_size_str()
        )
        .unwrap();
    }

    s.push_str("  ›  ");
    s.push_str(state.current_page.into());

    match state.current_page {
        Page::Games if let GamesState::Loaded(games) = &state.games => {
            write!(&mut s, "  (x{}", games.count()).unwrap();
            if let DriveState::Loaded(info) = &state.drive {
                write!(&mut s, "  ~{}", info.games_size_str()).unwrap();
            }
            s.push(')');
        }
        Page::HomebrewApps if let HomebrewState::Loaded(apps) = &state.homebrew => {
            write!(&mut s, "  (x{}", apps.count()).unwrap();
            if let DriveState::Loaded(info) = &state.drive {
                write!(&mut s, "  ~{}", info.apps_size_str()).unwrap();
            }
            s.push(')');
        }
        _ => {}
    }

    s
}
