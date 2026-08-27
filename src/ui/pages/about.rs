// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    messages::Message,
    state::AppState,
    ui::components::{my_card::my_card, my_link::my_link},
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

pub fn about(state: &AppState) -> Element<'_, Message> {
    let icon_handle = image::Handle::from_bytes(ICON_BYTES);

    let mq1_link = my_link("mq1", || "https://github.com/mq1", None);
    let license_link = my_link(
        "GPL-3.0-only",
        || "https://www.gnu.org/licenses/gpl-3.0.html",
        Icon::Scale,
    );
    let encounter_link = my_link(
        "Luke Street",
        || "https://github.com/encounter",
        Icon::Triangle,
    );
    let blackb0x_link = my_link("blackb0x", || "https://github.com/wiidev", Icon::Wand2);
    let data_dir_link = my_link("Data directory", || &state.data_dir, Icon::Folder);
    let source_code_link = my_link(
        "Source code",
        || "https://github.com/mq1/TinyWiiBackupManager",
        None,
    );
    let wiki_link = my_link(
        "Wiki",
        || "https://github.com/mq1/TinyWiiBackupManager/wiki",
        None,
    );

    column![
        space::vertical(),
        my_card(
            column![
                row![
                    image(icon_handle).width(128).height(128),
                    column![
                        space().height(20),
                        text(TITLE).size(20),
                        row![
                            "Copyright © 2026 Manuel Quarneti",
                            mq1_link,
                        ]
                        .spacing(40),
                        license_link,
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
                    encounter_link,
                    "for developing nod and helping TWBM leverage it effectively."
                ]
                .spacing(5),
                row![
                    blackb0x_link,
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
            data_dir_link,
            source_code_link,
            wiki_link,
        ]
        .padding(15)
        .spacing(10)
    ]
    .align_x(Alignment::Center)
    .width(Length::Fill)
    .into()
}
