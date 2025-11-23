// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use eframe::egui;
use egui_notify::{Anchor, Toasts};
use std::time::Duration;

pub struct Notifications(Toasts);

impl Default for Notifications {
    fn default() -> Self {
        Self::new()
    }
}

impl Notifications {
    pub fn new() -> Self {
        let toasts = Toasts::default()
            .with_anchor(Anchor::BottomRight)
            .with_margin(egui::vec2(8.0, 8.0))
            .with_shadow(egui::Shadow {
                offset: [0, 0],
                blur: 0,
                spread: 1,
                color: egui::Color32::GRAY,
            });

        Self(toasts)
    }

    pub fn show_err(&mut self, e: anyhow::Error) {
        log::error!("{:?}", e);
        self.0
            .error(format!("{e:#}"))
            .duration(Duration::from_secs(10));
    }

    pub fn show_success(&mut self, s: &str) {
        log::info!("{}", s);
        self.0.success(s).duration(Duration::from_secs(10));
    }

    pub fn show_info(&mut self, i: &str) {
        log::info!("{}", i);
        self.0.info(i);
    }

    pub fn show_info_no_duration(&mut self, i: &str) {
        log::info!("{}", i);
        self.0.info(i).duration(None);
    }

    pub fn show_toasts(&mut self, ctx: &egui::Context) {
        self.0.show(ctx);
    }
}
