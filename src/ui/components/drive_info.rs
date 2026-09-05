// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    messages::Message,
    state::AppState,
    ui::components::{my_card::my_card, my_link::my_link},
    util::drive_info::DriveInfo,
};
use iced::{
    Element,
    widget::{column, row, rule, space, text},
};
use lucide_icons::{Icon, iced::icon_hard_drive};
use size::Size;
use which_fs::FsKind;

fn fs_info(state: &AppState) -> (bool, Icon, &'static str) {
    match state.drive_info.as_ref().map(|info| info.fs_kind) {
        Some(FsKind::Fat32) => (
            true,
            Icon::Check,
            "optimal choice for game backups and homebrew apps",
        ),
        Some(FsKind::Ntfs | FsKind::ExFat) => (
            false,
            Icon::TriangleAlert,
            "limited support for game backups and homebrew apps",
        ),
        None | Some(FsKind::Unknown) => (false, Icon::TriangleAlert, "unknown support"),
        _ => (
            false,
            Icon::TriangleAlert,
            "won't work for game backups and homebrew apps",
        ),
    }
}

fn has_optimal_allocation_granularity(drive_info: &DriveInfo) -> bool {
    let is_small = drive_info.total_size <= Size::from_gib(32);
    let optimal = Size::from_kib(if is_small { 32 } else { 64 });
    optimal == drive_info.allocation_granularity
}

fn allocation_granularity(state: &AppState) -> (bool, Icon, &'static str) {
    match state
        .drive_info
        .as_ref()
        .map(has_optimal_allocation_granularity)
    {
        Some(true) => (true, Icon::Check, "optimal"),
        Some(false) => (false, Icon::TriangleAlert, "not optimal, but should work"),
        None => (false, Icon::TriangleAlert, "unknown"),
    }
}

pub fn drive_info(state: &AppState) -> Element<'_, Message> {
    let (optimal_fs_info, fs_info_icon, fs_info_comment) = fs_info(state);
    let (
        optimal_allocation_granularity,
        allocation_granularity_icon,
        allocation_granularity_comment,
    ) = allocation_granularity(state);

    my_card(
        column![
            row![
                icon_hard_drive(),
                "Drive info",
                space::horizontal(),
                (!optimal_fs_info || !optimal_allocation_granularity).then(|| my_link(
                    "Drive formatting guide",
                    || "https://gbatemp.net/threads/usb-loader-gx-troubleshooting-guide.617564/#fs",
                    None
                ))
            ]
            .spacing(5),
            rule::horizontal(1),
            row![
                fs_info_icon.widget(),
                text!(
                    "Filesystem: {}  ({})",
                    state
                        .drive_info
                        .as_ref()
                        .map(|info| info.fs_kind)
                        .unwrap_or(FsKind::Unknown),
                    fs_info_comment
                )
            ]
            .spacing(5),
            row![
                allocation_granularity_icon.widget(),
                text!(
                    "Allocation granularity: {}  ({})",
                    state
                        .drive_info
                        .as_ref()
                        .map(|info| info.allocation_granularity)
                        .unwrap_or_default(),
                    allocation_granularity_comment
                )
            ]
            .spacing(5),
        ]
        .spacing(10)
        .padding(5),
    )
    .padding(5)
    .into()
}
