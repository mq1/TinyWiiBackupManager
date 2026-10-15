// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    message::Message,
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
use lucide_iced::{
    bytes::{CHECK, HARD_DRIVE, TRIANGLE_ALERT},
    themed_icon,
};
use which_fs::FsKind;

fn fs_info(info: &DriveInfo) -> (bool, &'static [u8], &'static str) {
    match info.fs_kind() {
        FsKind::Fat32 => (
            true,
            CHECK,
            "optimal choice for game backups and homebrew apps",
        ),
        FsKind::Ntfs | FsKind::ExFat => (
            false,
            TRIANGLE_ALERT,
            "limited support for game backups and homebrew apps",
        ),
        FsKind::Unknown => (false, TRIANGLE_ALERT, "unknown support"),
        _ => (
            false,
            TRIANGLE_ALERT,
            "won't work for game backups and homebrew apps",
        ),
    }
}

fn allocation_granularity(info: &DriveInfo) -> (bool, &'static [u8], &'static str) {
    if info.has_optimal_allocation_granularity() {
        (true, CHECK, "optimal")
    } else {
        (false, TRIANGLE_ALERT, "not optimal, but should work")
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
                        themed_icon(fs_info_icon, 14.),
                        text!("Filesystem: {}  ({})", info.fs_kind(), fs_info_comment)
                    ]
                    .spacing(5),
                    row![
                        themed_icon(allocation_granularity_icon, 14.),
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
        .icon(HARD_DRIVE)
        .top_right_content(top_right_content)
        .into()
}
