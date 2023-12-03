// SPDX-FileCopyrightText: 2023 Manuel Quarneti <manuelquarneti@protonmail.com>
// SPDX-License-Identifier: GPL-2.0-only

use eframe::egui;
use poll_promise::Promise;
use anyhow::Result;

use crate::pages::{Page, self};
use crate::types::drive::Drive;
use crate::types::game::Game;


pub struct App {
    pub page: Page,
    pub drives: Option<Promise<Vec<Drive>>>,
    pub current_drive: Option<Drive>,
    pub games: Option<Promise<Result<Vec<(Game, bool)>>>>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            page: Page::Drives,
            drives: None,
            current_drive: None,
            games: None,
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