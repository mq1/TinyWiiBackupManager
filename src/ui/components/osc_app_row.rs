// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{message::Message, osc::osc_app::OscApp, ui::components::row_button::row_button};
use iced::{
    Element, Length, padding,
    widget::{row, rule, space, text},
};
use lucide_iced::bytes::{CLOUD_DOWNLOAD, INFO, MONITOR_UP};

pub fn osc_app_row(app: &OscApp) -> Element<'_, Message> {
    row![
        text!("{} ({})", app.name(), app.version()),
        space::horizontal(),
        text(app.size_str()),
        rule::vertical(1),
        row_button(
            CLOUD_DOWNLOAD,
            "Install",
            Some(|| Message::InstallOscApp(app.clone()))
        ),
        row_button(
            INFO,
            "App info",
            Some(|| Message::OpenOscAppInfo(app.clone()))
        ),
        row_button(
            MONITOR_UP,
            "Send via Wiiload",
            Some(|| Message::SendOscAppViaWiiload(app.clone()))
        ),
    ]
    .spacing(5)
    .height(Length::Shrink)
    .padding(padding::all(2).left(5))
    .into()
}
