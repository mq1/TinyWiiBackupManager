// SPDX-FileCopyrightText: 2023 Manuel Quarneti <manuelquarneti@protonmail.com>
// SPDX-License-Identifier: GPL-2.0-only

use anyhow::Result;
use eframe::egui;
use poll_promise::Promise;
use rfd::{MessageButtons, MessageDialog};
use std::sync::{Arc, Mutex};

use crate::pages::{self, Page};
use crate::types::drive::Drive;
use crate::types::game::Game;
use crate::updater::check_for_updates;

pub struct App {
    pub page: Page,
    pub drives: Option<Promise<Vec<Drive>>>,
    pub current_drive: Option<Drive>,
    pub games: Option<Promise<Result<Vec<Game>>>>,
    pub adding_games_progress: Arc<Mutex<Option<(usize, usize)>>>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            page: Page::Drives,
            drives: None,
            current_drive: None,
            games: None,
            adding_games_progress: Arc::new(Mutex::new(None)),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                if ui.button("ℹ About").clicked() {
                    let desc = format!(
                        "v{}\n{}\n\nCopyright (c) 2023 {}\n{} Licensed",
                        env!("CARGO_PKG_VERSION"),
                        env!("CARGO_PKG_DESCRIPTION"),
                        env!("CARGO_PKG_AUTHORS"),
                        env!("CARGO_PKG_LICENSE")
                    );
                    MessageDialog::new()
                        .set_title(env!("CARGO_PKG_NAME"))
                        .set_description(desc)
                        .set_buttons(MessageButtons::Ok)
                        .show();
                }

                ui.add_space(10.0);

                if ui.button("♻ Check for updates").clicked() {
                    check_for_updates().unwrap();
                }
            });
        });

        match self.page {
            Page::Drives => pages::drives::view(ctx, self),
            Page::Games => pages::games::view(ctx, self),
            Page::AddingGames => pages::adding_games::view(ctx, self),
        }
    }
}
