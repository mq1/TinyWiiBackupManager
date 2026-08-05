// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{messages::Message, state::AppState, ui::components};
use iced::{
    Alignment, Element, Length,
    widget::{column, image, row, rule, space, text},
};
use lucide_icons::{
    Icon,
    iced::{icon_alert_triangle, icon_heart},
};

const ICON_BYTES: &[u8] = include_bytes!("../../../assets/TinyWiiBackupManager-256x256.png");
const TITLE: &str = concat!("TinyWiiBackupManager v", env!("CARGO_PKG_VERSION"));

pub fn view(state: &AppState) -> Element<'_, Message> {
    let icon_handle = image::Handle::from_bytes(ICON_BYTES);

    column![
        space::vertical(),
        components::card::view(
            column![
                row![
                    image(icon_handle).width(128).height(128),
                    column![
                        space().height(20),
                        text(TITLE).size(20),
                        row![
                            "Copyright © 2026 Manuel Quarneti",
                            components::link::view("github/mq1", None, || "https://github.com/mq1"),
                        ]
                        .spacing(40),
                        components::link::view(
                            "GPL-3.0-only",
                            None,
                            || "https://www.gnu.org/licenses/gpl-3.0.html"
                        )
                    ]
                    .spacing(5)
                ]
                .spacing(20),
                row![
                    icon_alert_triangle().size(20),
                    "TinyWiiBackupManager is intended strictly for legal homebrew use and is not affiliated with or endorsed by Nintendo. Use of TinyWiiBackupManager for pirated or unauthorized copies of games is strictly prohibited."
                ]
                .spacing(20)
                .align_y(Alignment::Center),
                rule::horizontal(1),
                row![
                    icon_heart().size(16),
                    text("Special thanks to").size(16)
                ]
                .spacing(5),
                row![
                    components::link::view(
                        "Luke Street",
                        Some(Icon::Triangle),
                        || "https://github.com/encounter"
                    ),
                    "for developing nod and helping TWBM leverage it effectively."
                ]
                .spacing(5),
                row![
                    components::link::view(
                        "blackb0x",
                        Some(Icon::Wand2),
                        || "https://github.com/wiidev"
                    ),
                    "for invaluable feedback and advice during TWBM's development."
                ]
                .spacing(5),
            ]
            .padding(20)
            .spacing(10)
        )
        .width(600),
        space::vertical(),
        row![
            space::horizontal(),
            components::link::view("Data directory", Some(Icon::Folder), || &state.data_dir),
            components::link::view("Source code", None, || "https://github.com/mq1/TinyWiiBackupManager"),
            components::link::view("Wiki", None, || "https://github.com/mq1/TinyWiiBackupManager/wiki")
        ]
        .padding(15)
        .spacing(10)
    ]
    .align_x(Alignment::Center)
    .width(Length::Fill)
    .into()
}
