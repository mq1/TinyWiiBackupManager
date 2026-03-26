// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use slint::Window;
use std::path::PathBuf;

#[cfg(not(target_vendor = "win7"))]
use rfd::FileDialog;

#[cfg(target_vendor = "win7")]
use crate::ui::xp_dialogs;

pub fn pick_mount_point(window: &Window) -> Option<PathBuf> {
    #[cfg(not(target_vendor = "win7"))]
    let res = FileDialog::new()
        .set_parent(&window.window_handle())
        .set_title("Select Drive/Mount Point")
        .pick_folder();

    #[cfg(target_vendor = "win7")]
    let res = xp_dialogs::pick_dir("Select Drive/Mount Point");

    res
}
