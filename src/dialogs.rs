// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    Game,
    extensions::{INPUT_DIALOG_FILTER, OUTPUT_DIALOG_FILTER},
    util,
};
use slint::Window;
use std::path::PathBuf;
use walkdir::WalkDir;

#[cfg(unix)]
use rfd::FileDialog;

#[cfg(windows)]
use crate::xp_dialogs;

pub fn pick_mount_point(window: &Window) -> Option<PathBuf> {
    #[cfg(unix)]
    let res = FileDialog::new()
        .set_parent(&window.window_handle())
        .set_title("Select Drive/Mount Point")
        .pick_folder();

    #[cfg(windows)]
    let res = xp_dialogs::pick_dir(&window.window_handle(), "Select Drive/Mount Point");

    res
}

pub fn pick_games(window: &Window) -> Vec<PathBuf> {
    #[cfg(unix)]
    let paths = FileDialog::new()
        .set_parent(&window.window_handle())
        .set_title("Select Games")
        .add_filter(INPUT_DIALOG_FILTER.0, INPUT_DIALOG_FILTER.1)
        .pick_files()
        .unwrap_or_default();

    #[cfg(windows)]
    let paths = xp_dialogs::pick_files("Select Games", INPUT_DIALOG_FILTER);

    paths
}

pub fn pick_games_r(window: &Window) -> Vec<PathBuf> {
    #[cfg(unix)]
    let res = FileDialog::new()
        .set_parent(&window.window_handle())
        .set_title("Select folder (games will be searched recursively)")
        .pick_folder();

    #[cfg(windows)]
    let res = xp_dialogs::pick_dir(
        &window.window_handle(),
        "Select folder (games will be searched recursively)",
    );

    let mut paths = Vec::new();

    let Some(res) = res else {
        return paths;
    };

    for entry in WalkDir::new(res).into_iter().filter_map(Result::ok) {
        if entry.file_type().is_file()
            && let Some(ext) = entry.path().extension()
            && INPUT_DIALOG_FILTER
                .1
                .iter()
                .any(|e| ext.eq_ignore_ascii_case(e))
        {
            paths.push(entry.into_path());
        }
    }

    paths
}

pub fn save_game(window: &Window, game: &Game) -> Option<PathBuf> {
    let title = format!(
        "Select Destination for {} | Supported extensions: iso, wbfs, wia, rvz, ciso, gcz, tgc, nfs",
        &game.title
    );

    let filename = format!("{}.rvz", util::sanitize(&game.title));

    #[cfg(unix)]
    let res = FileDialog::new()
        .set_parent(&window.window_handle())
        .set_title(title)
        .set_file_name(filename)
        .add_filter(OUTPUT_DIALOG_FILTER.0, OUTPUT_DIALOG_FILTER.1)
        .save_file();

    #[cfg(windows)]
    let res = xp_dialogs::save_file(&title, OUTPUT_DIALOG_FILTER, &filename);

    res
}
