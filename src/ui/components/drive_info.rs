// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    messages::Message,
    state::AppState,
    ui::components::{
        my_group::my_group,
        my_link::{MyLink, my_link},
    },
    util::drive_state::{DriveInfo, DriveState},
};
use iced::{
    Element,
    widget::{column, row, text},
};
use lucide_icons::Icon;
use which_fs::FsKind;

fn fs_info(info: &DriveInfo) -> (bool, Icon, &'static str) {
    match info.fs_kind() {
        FsKind::Fat32 => (
            true,
            Icon::Check,
            "optimal choice for game backups and homebrew apps",
        ),
        FsKind::Ntfs | FsKind::ExFat => (
            false,
            Icon::TriangleAlert,
            "limited support for game backups and homebrew apps",
        ),
        FsKind::Unknown => (false, Icon::TriangleAlert, "unknown support"),
        _ => (
            false,
            Icon::TriangleAlert,
            "won't work for game backups and homebrew apps",
        ),
    }
}

fn allocation_granularity(info: &DriveInfo) -> (bool, Icon, &'static str) {
    if info.has_optimal_allocation_granularity() {
        (true, Icon::Check, "optimal")
    } else {
        (false, Icon::TriangleAlert, "not optimal, but should work")
    }
}

fn drive_formatting_guide_link() -> MyLink<&'static str, &'static str> {
    my_link(
        "Drive formatting guide",
        "https://gbatemp.net/threads/usb-loader-gx-troubleshooting-guide.617564/#fs",
    )
}

pub fn drive_info(state: &AppState) -> Element<'_, Message> {
    let (content, top_right_content): (Element<'_, Message>, Element<'_, Message>) =
        match &state.drive {
            DriveState::NotLoaded | DriveState::Loading => (
                text("Loading...").into(),
                None::<Element<'_, Message>>.into(),
            ),
            DriveState::Errored(msg) => (
                text(msg.as_str()).into(),
                None::<Element<'_, Message>>.into(),
            ),
            DriveState::Loaded(info) => {
                let (optimal_fs_info, fs_info_icon, fs_info_comment) = fs_info(info);
                let (
                    optimal_allocation_granularity,
                    allocation_granularity_icon,
                    allocation_granularity_comment,
                ) = allocation_granularity(info);

                let top_right_content = (!optimal_fs_info || !optimal_allocation_granularity)
                    .then(drive_formatting_guide_link)
                    .into();

                let content = column![
                    row![
                        fs_info_icon.widget(),
                        text!("Filesystem: {}  ({})", info.fs_kind(), fs_info_comment)
                    ]
                    .spacing(5),
                    row![
                        allocation_granularity_icon.widget(),
                        text!(
                            "Allocation granularity: {}  ({})",
                            info.allocation_granularity_str(),
                            allocation_granularity_comment
                        )
                    ]
                    .spacing(5),
                ]
                .spacing(5)
                .into();

                (content, top_right_content)
            }
        };

    my_group("Drive info", content)
        .icon(Icon::HardDrive)
        .top_right_content(top_right_content)
        .into()
}
