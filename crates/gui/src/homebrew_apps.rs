// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{DisplayedHomebrewApp, DisplayedOscApp, util::MIB};
use slint::{Image, Rgba8Pixel, SharedPixelBuffer, ToSharedString};
use std::path::Path;
use twbm_core::homebrew_app::HomebrewApp;

impl DisplayedHomebrewApp {
    #[must_use]
    pub fn new(app: &HomebrewApp, osc_app: DisplayedOscApp) -> Self {
        let buffer = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
            app.icon_rgba8.as_raw(),
            app.icon_rgba8.width(),
            app.icon_rgba8.height(),
        );
        let icon = Image::from_rgba8(buffer);

        let slug = app.path.file_name().unwrap_or_default().to_string_lossy();

        Self {
            slug: slug.to_shared_string(),
            path: app.path.to_string_lossy().to_shared_string(),
            size_mib: app.size as f32 / MIB,
            icon,
            name: app.meta.name.to_shared_string(),
            coder: app.meta.coder.to_shared_string(),
            version: app.meta.version.to_shared_string(),
            release_date: app.meta.release_date.to_shared_string(),
            short_description: app.meta.short_description.to_shared_string(),
            long_description: app.meta.long_description.to_shared_string(),
            osc_app,
        }
    }
}

pub fn scan_drive(root_path: &Path) -> Vec<HomebrewApp> {
    let apps_dir = root_path.join("apps");
    twbm_core::homebrew_app::scan_dir(&apps_dir)
}
