// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use rfd::FileDialog;
use std::path::PathBuf;

pub fn pick_mount_point(w: &dyn iced::Window) -> Option<PathBuf> {
    FileDialog::new()
        .set_parent(w)
        .set_title("Select Drive/Mount Point")
        .pick_folder()
}
