// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{message::Message, ui::components::row_button::row_button};
use iced::{
    Element, Length, padding,
    widget::{row, rule, space, text},
};
use lucide_icons::Icon;
use twbm_osc::app::OscApp;

pub fn osc_app_row(app: &OscApp) -> Element<'_, Message> {
    row![
        text!("{} ({})", app.name(), app.version()),
        space::horizontal(),
        text!("{}", app.size()),
        rule::vertical(1),
        row_button(
            Icon::CloudDownload,
            "Install",
            Some(|| Message::InstallOscApp(app.clone()))
        ),
        row_button(
            Icon::Info,
            "App info",
            Some(|| Message::OpenOscAppInfo(app.clone()))
        ),
        row_button(
            Icon::MonitorUp,
            "Send via Wiiload",
            Some(|| Message::SendOscAppViaWiiload(app.clone()))
        ),
    ]
    .spacing(5)
    .height(Length::Shrink)
    .padding(padding::all(2).left(5))
    .into()
}
