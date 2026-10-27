// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    message::Message,
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
use std::sync::LazyLock;

const ICON_BYTES: &[u8] = include_bytes!("../../../assets/UltimateGameBackupManager-256x256.png");
const TITLE: &str = concat!("Ultimate Game Backup Manager v", env!("CARGO_PKG_VERSION"));

static ICON: LazyLock<image::Handle> = LazyLock::new(|| image::Handle::from_bytes(ICON_BYTES));

pub fn about(state: &AppState) -> Element<'_, Message> {
    let petter1_link = my_link("petter1", "https://github.com/petter1");
    let mq1_link = my_link("mq1", "https://github.com/mq1").icon(Icon::Heart);
    let license_link =
        my_link("GPL-3.0-only", "https://www.gnu.org/licenses/gpl-3.0.html").icon(Icon::Scale);
    let encounter_link =
        my_link("Luke Street", "https://github.com/encounter").icon(Icon::Triangle);
    let blackb0x_link = my_link("blackb0x", "https://github.com/wiidev").icon(Icon::Wand2);
    let data_dir_link = my_link("Data directory", &state.data_dir).icon(Icon::Folder);
    let source_code_link = my_link(
        "Source code",
        "https://github.com/petter1/UltimateGameBackupManager",
    );
    let upstream_link = my_link(
        "TinyWiiBackupManager",
        "https://github.com/mq1/TinyWiiBackupManager",
    );

    column![
        space::vertical(),
        my_card(
            column![
                row![
                    image(&*ICON).width(128).height(128),
                    column![
                        space().height(20),
                        text(TITLE).size(20),
                        row!["Copyright © 2026 Yanik Fuchs", petter1_link].spacing(40),
                        row!["Based on TinyWiiBackupManager © 2026 Manuel Quarneti"]
                            .spacing(40),
                        license_link,
                    ]
                    .spacing(5)
                ]
                .spacing(20),
                row![
                    icon_alert_triangle().size(20),
                    "Ultimate Game Backup Manager is intended strictly for legal homebrew use and is not affiliated with or endorsed by Nintendo. Use of Ultimate Game Backup Manager for pirated or unauthorized copies of games is strictly prohibited."
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
                    mq1_link,
                    "for TinyWiiBackupManager, the app this one is built on."
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
                rule::horizontal(1),
                text("Written with the help of AI (Claude). See the README for details.").size(12),
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
            upstream_link,
        ]
        .padding(15)
        .spacing(10)
    ]
    .align_x(Alignment::Center)
    .width(Length::Fill)
    .into()
}
