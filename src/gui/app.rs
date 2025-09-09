// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::app::{App, View};
use crate::gui;
use crate::messages::handle_messages;
use eframe::egui::ViewportCommand;
use eframe::{Storage, egui};
use size::Size;

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        handle_messages(self, ctx);

        // Update window title
        let dir_text = if let Some(base_dir) = &self.base_dir {
            match self.view {
                View::Games => format!(
                    " • {} games in {} ({}/{})",
                    self.games.len(),
                    base_dir.name(),
                    Size::from_bytes(self.used_space),
                    Size::from_bytes(self.total_space),
                ),
                View::WiiApps => format!(
                    " • {} apps in {} ({}/{})",
                    self.wiiapps.len(),
                    base_dir.name(),
                    Size::from_bytes(self.used_space),
                    Size::from_bytes(self.total_space),
                ),
            }
        } else {
            String::new()
        };

        ctx.send_viewport_cmd(ViewportCommand::Title(format!(
            "TinyWiiBackupManager{dir_text}"
        )));

        gui::top_panel::ui_top_panel(ctx, self);
        gui::bottom_panel::ui_bottom_panel(ctx, self);

        egui::CentralPanel::default().show(ctx, |ui| match self.view {
            View::Games => gui::game_grid::ui_game_grid(ui, self),
            View::WiiApps => gui::wiiapps::ui_apps(ui, self),
        });

        gui::settings::ui_settings_window(ctx, self);
        gui::oscwii_window::ui_oscwii_window(ctx, self);

        self.top_right_toasts.show(ctx);
        self.bottom_left_toasts.show(ctx);
        self.bottom_right_toasts.show(ctx);
    }

    fn save(&mut self, storage: &mut dyn Storage) {
        eframe::set_value(storage, "app_version", &APP_VERSION);
        eframe::set_value(storage, "settings", &self.settings);
        eframe::set_value(storage, "oscwii_contents", &self.oscwii_apps);

        if let Some(base_dir) = &self.base_dir {
            eframe::set_value(storage, "base_dir", base_dir);
        }
    }
}
