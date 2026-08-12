// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    messages::Message,
    state::AppState,
    ui::components::{my_card::MyCard, my_link::MyLink},
};
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

    let mq1_link = MyLink::new("mq1", "https://github.com/mq1");
    let license_link =
        MyLink::new("GPL-3.0-only", "https://www.gnu.org/licenses/gpl-3.0.html").icon(Icon::Scale);
    let encounter_link =
        MyLink::new("Luke Street", "https://github.com/encounter").icon(Icon::Triangle);
    let blackb0x_link = MyLink::new("blackb0x", "https://github.com/wiidev").icon(Icon::Wand2);
    let data_dir_link = MyLink::new("data directory", &state.data_dir).icon(Icon::Folder);
    let source_code_link =
        MyLink::new("source code", "https://github.com/mq1/TinyWiiBackupManager");
    let wiki_link = MyLink::new("wiki", "https://github.com/mq1/TinyWiiBackupManager/wiki");

    column![
        space::vertical(),
        MyCard::new(
            column![
                row![
                    image(icon_handle).width(128).height(128),
                    column![
                        space().height(20),
                        text(TITLE).size(20),
                        row![
                            "Copyright © 2026 Manuel Quarneti",
                            mq1_link.view(),
                        ]
                        .spacing(40),
                        license_link.view(),
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
                    encounter_link.view(),
                    "for developing nod and helping TWBM leverage it effectively."
                ]
                .spacing(5),
                row![
                    blackb0x_link.view(),
                    "for invaluable feedback and advice during TWBM's development."
                ]
                .spacing(5),
            ]
            .padding(20)
            .spacing(10)
        )
        .view()
        .width(600),
        space::vertical(),
        row![
            space::horizontal(),
            data_dir_link.view(),
            source_code_link.view(),
            wiki_link.view(),
        ]
        .padding(15)
        .spacing(10)
    ]
    .align_x(Alignment::Center)
    .width(Length::Fill)
    .into()
}
