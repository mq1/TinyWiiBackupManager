// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use eframe::egui;
use eframe::egui::{Align2, Color32, Direction, Order, WidgetText};
use egui_phosphor::regular as ph;
use egui_toast::{Toast, ToastKind, ToastOptions, ToastStyle, Toasts};

pub struct Notifications {
    toasts: Toasts,
    style: ToastStyle,
}

impl Default for Notifications {
    fn default() -> Self {
        Self::new()
    }
}

impl Notifications {
    pub fn new() -> Self {
        let toasts = Toasts::new()
            .anchor(Align2::RIGHT_BOTTOM, (-10., -10.))
            .direction(Direction::BottomUp)
            .order(Order::Tooltip);

        let style = ToastStyle {
            info_icon: WidgetText::from(ph::INFO).color(Color32::from_rgb(0, 155, 255)),
            warning_icon: WidgetText::from(ph::WARNING).color(Color32::from_rgb(255, 212, 0)),
            error_icon: WidgetText::from(ph::WARNING).color(Color32::from_rgb(255, 32, 0)),
            success_icon: WidgetText::from(ph::SEAL_CHECK).color(Color32::from_rgb(0, 255, 32)),
            close_button_text: WidgetText::from(ph::X),
        };

        Self { toasts, style }
    }

    pub fn show_err(&mut self, e: anyhow::Error) {
        log::error!("{:?}", e);

        self.toasts.add(Toast {
            text: format!("{:#}", e).into(),
            kind: ToastKind::Error,
            options: ToastOptions::default(),
            style: self.style.clone(),
        });
    }

    pub fn show_success(&mut self, s: &str) {
        log::info!("{}", s);

        self.toasts.add(Toast {
            text: s.into(),
            kind: ToastKind::Success,
            options: ToastOptions::default(),
            style: self.style.clone(),
        });
    }

    pub fn show_info(&mut self, i: &str) {
        log::info!("{}", i);

        self.toasts.add(Toast {
            text: i.into(),
            kind: ToastKind::Info,
            options: ToastOptions::default().duration_in_seconds(5.0),
            style: self.style.clone(),
        });
    }

    pub fn show_info_no_duration(&mut self, i: &str) {
        log::info!("{}", i);

        self.toasts.add(Toast {
            text: i.into(),
            kind: ToastKind::Success,
            options: ToastOptions::default(),
            style: self.style.clone(),
        });
    }

    pub fn show_toasts(&mut self, ctx: &egui::Context) {
        self.toasts.show(ctx);
    }
}
