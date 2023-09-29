// SPDX-FileCopyrightText: 2023 Manuel Quarneti <manuel.quarneti@proton.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::types::app::App;

mod wbfs_file;
mod types;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "wbfs-manager-rs",
        native_options,
        Box::new(|cc| Box::new(App::new(cc))),
    )
}
