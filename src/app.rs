// SPDX-FileCopyrightText: 2023 Manuel Quarneti <manuelquarneti@protonmail.com>
// SPDX-License-Identifier: GPL-2.0-only

use anyhow::Result;
use eframe::egui;
use poll_promise::Promise;
use std::sync::{Arc, Mutex};

use crate::pages::{self, Page};
use crate::types::drive::Drive;
use crate::types::game::Game;

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
        match self.page {
            Page::Drives => pages::drives::view(ctx, self),
            Page::Games => pages::games::view(ctx, self),
        }
    }
}
