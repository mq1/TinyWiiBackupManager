// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{config::TxtCodesSource, message::Message, state::State, ui::style};
use iced::{
    Element, Length,
    widget::{column, container, radio, row, rule, scrollable, space, text},
};
use lucide_icons::iced::{
    icon_disc_3, icon_file_archive, icon_file_minus_corner, icon_settings, icon_skull,
    icon_square_split_horizontal, icon_trash,
};

#[allow(clippy::too_many_lines)]
pub fn view(state: &State) -> Element<'_, Message> {
    let wii_output_format = state.config.wii_output_format();
    let gc_output_format = state.config.gc_output_format();
    let archive_format = state.config.archive_format();
    let always_split = state.config.always_split();
    let scrub_update_partition = state.config.scrub_update_partition();
    let remove_sources_games = state.config.remove_sources_games();
    let remove_sources_apps = state.config.remove_sources_apps();
    let txt_codes_source = state.config.txt_codes_source();

    column![
        row![icon_settings().size(18), text("Settings").size(18)]
            .spacing(5)
            .padding(10),
        scrollable(
            column![
                container(
                    column![
                        row![icon_disc_3(), text("Wii output format")].spacing(5),
                        rule::horizontal(1),
                        space(),
                        radio(
                            "WBFS (recommended)",
                            nod::common::Format::Wbfs,
                            Some(wii_output_format),
                            |format| Message::UpdateConfig(
                                state.config.clone_with_wii_output_format(format)
                            )
                        ),
                        radio(
                            "ISO (very large)",
                            nod::common::Format::Iso,
                            Some(wii_output_format),
                            |format| Message::UpdateConfig(
                                state.config.clone_with_wii_output_format(format)
                            )
                        ),
                    ]
                    .spacing(5)
                    .padding(10)
                    .width(Length::Fill)
                )
                .style(style::card),
                container(
                    column![
                        row![icon_disc_3(), text("GameCube output format")].spacing(5),
                        rule::horizontal(1),
                        space(),
                        radio(
                            "ISO (recommended)",
                            nod::common::Format::Iso,
                            Some(gc_output_format),
                            |format| Message::UpdateConfig(
                                state.config.clone_with_gc_output_format(format)
                            )
                        ),
                        radio(
                            "CISO (much smaller, slightly slower, less metadata in game loaders)",
                            nod::common::Format::Ciso,
                            Some(gc_output_format),
                            |format| Message::UpdateConfig(
                                state.config.clone_with_gc_output_format(format)
                            )
                        ),
                    ]
                    .spacing(5)
                    .padding(10)
                    .width(Length::Fill)
                )
                .style(style::card),
                container(
                    column![
                        row![icon_file_archive(), text("Archive format")].spacing(5),
                        rule::horizontal(1),
                        space(),
                        radio(
                            "RVZ zstd-19 (recommended)",
                            nod::common::Format::Rvz,
                            Some(archive_format),
                            |format| Message::UpdateConfig(
                                state.config.clone_with_archive_format(format)
                            )
                        ),
                        radio(
                            "ISO (very large)",
                            nod::common::Format::Iso,
                            Some(archive_format),
                            |format| Message::UpdateConfig(
                                state.config.clone_with_archive_format(format)
                            )
                        ),
                    ]
                    .spacing(5)
                    .padding(10)
                    .width(Length::Fill)
                )
                .style(style::card),
                container(
                    column![
                        row![icon_square_split_horizontal(), text("Split output")].spacing(5),
                        rule::horizontal(1),
                        space(),
                        radio(
                            "Only when needed (recommended)",
                            false,
                            Some(always_split),
                            |yes| Message::UpdateConfig(state.config.clone_with_always_split(yes))
                        ),
                        radio("Always 4GB-32KB", true, Some(always_split), |yes| {
                            Message::UpdateConfig(state.config.clone_with_always_split(yes))
                        }),
                    ]
                    .spacing(5)
                    .padding(10)
                    .width(Length::Fill)
                )
                .style(style::card),
                container(
                    column![
                        row![
                            icon_file_minus_corner(),
                            text("Remove update partition on WBFS/CISO")
                        ]
                        .spacing(5),
                        rule::horizontal(1),
                        space(),
                        radio(
                            "No (recommended)",
                            false,
                            Some(scrub_update_partition),
                            |yes| Message::UpdateConfig(
                                state.config.clone_with_scrub_update_partition(yes)
                            )
                        ),
                        radio(
                            "Yes (saves some space; update partition is zeroed, but still there)",
                            true,
                            Some(scrub_update_partition),
                            |yes| Message::UpdateConfig(
                                state.config.clone_with_scrub_update_partition(yes)
                            )
                        ),
                    ]
                    .spacing(5)
                    .padding(10)
                    .width(Length::Fill)
                )
                .style(style::card),
                container(
                    column![
                        row![icon_trash(), text("Delete sources when adding games")].spacing(5),
                        rule::horizontal(1),
                        space(),
                        radio(
                            "No (recommended)",
                            false,
                            Some(remove_sources_games),
                            |yes| Message::UpdateConfig(
                                state.config.clone_with_remove_sources_games(yes)
                            )
                        ),
                        radio("Yes", true, Some(remove_sources_games), |yes| {
                            Message::UpdateConfig(state.config.clone_with_remove_sources_games(yes))
                        }),
                    ]
                    .spacing(5)
                    .padding(10)
                    .width(Length::Fill)
                )
                .style(style::card),
                container(
                    column![
                        row![icon_trash(), text("Delete sources when adding apps")].spacing(5),
                        rule::horizontal(1),
                        space(),
                        radio(
                            "No (recommended)",
                            false,
                            Some(remove_sources_apps),
                            |yes| Message::UpdateConfig(
                                state.config.clone_with_remove_sources_apps(yes)
                            )
                        ),
                        radio("Yes", true, Some(remove_sources_apps), |yes| {
                            Message::UpdateConfig(state.config.clone_with_remove_sources_apps(yes))
                        }),
                    ]
                    .spacing(5)
                    .padding(10)
                    .width(Length::Fill)
                )
                .style(style::card),
                container(
                    column![
                        row![icon_skull(), text("Cheat codes source")].spacing(5),
                        rule::horizontal(1),
                        space(),
                        radio(
                            "geckocodes.org (web.archive.org)    (Recommended, high quality)",
                            TxtCodesSource::WebArchive,
                            Some(txt_codes_source),
                            |source| Message::UpdateConfig(
                                state.config.clone_with_txt_codes_source(source)
                            )
                        ),
                        radio(
                            "codes.rc24.xyz    (Lower quality)",
                            TxtCodesSource::Rc24,
                            Some(txt_codes_source),
                            |source| Message::UpdateConfig(
                                state.config.clone_with_txt_codes_source(source)
                            )
                        ),
                        radio(
                            "gamehacking.org    (Up to date, download might fail)",
                            TxtCodesSource::GameHacking,
                            Some(txt_codes_source),
                            |source| Message::UpdateConfig(
                                state.config.clone_with_txt_codes_source(source)
                            )
                        ),
                    ]
                    .spacing(5)
                    .padding(10)
                    .width(Length::Fill)
                )
                .style(style::card)
            ]
            .spacing(10)
            .padding(10)
            .width(Length::Fill)
            .height(Length::Fill)
        )
        .spacing(1)
    ]
    .height(Length::Fill)
    .into()
}
