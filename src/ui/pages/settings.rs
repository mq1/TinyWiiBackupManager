// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::{GcOutputFormat, PreferredLanguage, ThemePreference, TxtCodesSource, WiiOutputFormat},
    messages::Message,
    state::AppState,
    ui::components::my_card::my_card,
};
use iced::{
    Element, Length, padding,
    widget::{Column, column, radio, row, rule, scrollable, text},
};
use lucide_icons::{Icon, iced::icon_chevron_right};
use strum::IntoEnumIterator;

fn setting<T: Eq + Copy>(
    label: &'static str,
    icon: Icon,
    items: impl IntoIterator<Item = (T, &'static str)>,
    active: T,
    on_change: fn(T) -> Message,
) -> Element<'static, Message> {
    let heading = row![icon.widget(), label].spacing(5);

    let contents = items
        .into_iter()
        .map(|(value, label)| radio(label, value, Some(active), on_change).into())
        .collect::<Column<'_, _>>()
        .spacing(5);

    my_card(
        column![heading, rule::horizontal(1), contents]
            .spacing(10)
            .padding(5),
    )
    .padding(5)
    .into()
}

pub fn settings(state: &AppState) -> Element<'_, Message> {
    let titlebar = row![icon_chevron_right().size(20), text("Settings").size(20)]
        .spacing(5)
        .padding(padding::all(14).left(10));

    let contents = scrollable(
        column![
            setting(
                "Wii output format",
                Icon::Disc3,
                [
                    (WiiOutputFormat::Wbfs, "WBFS (Recommended)"),
                    (WiiOutputFormat::Iso, "ISO (very large)"),
                ],
                state.config.wii_output_format,
                Message::SetWiiOutputFormat
            ),
            setting(
                "GameCube output format",
                Icon::Disc3,
                [
                    (GcOutputFormat::Iso, "ISO (Recommended)"),
                    (
                        GcOutputFormat::Ciso,
                        "CISO (much smaller, slightly slower, less metadata in game loaders)"
                    ),
                ],
                state.config.gc_output_format,
                Message::SetGcOutputFormat
            ),
            setting(
                "Split output",
                Icon::SquareSplitHorizontal,
                [
                    (false, "Only when needed (recommended)"),
                    (true, "Always 4GB-32KB"),
                ],
                state.config.always_split,
                Message::SetAlwaysSplit
            ),
            // setting(
            //     "Remove update partition on WBFS/CISO",
            //     Icon::FileMinusCorner,
            //     [
            //         (false, "No (recommended)"),
            //         (
            //             true,
            //             "Yes (saves some space; update partition is zeroed, but still there)"
            //         ),
            //     ],
            //     state.config.scrub_update_partition,
            //     Message::SetScrubUpdatePartition
            // ),
            setting(
                "Delete sources when adding games",
                Icon::Trash,
                [(false, "No (recommended)"), (true, "Yes")],
                state.config.remove_sources_games,
                Message::SetRemoveSourcesGames
            ),
            setting(
                "Delete sources when adding apps",
                Icon::Trash,
                [(false, "No (recommended)"), (true, "Yes")],
                state.config.remove_sources_apps,
                Message::SetRemoveSourcesApps
            ),
            setting(
                "Cheat codes source",
                Icon::Skull,
                [
                    (
                        TxtCodesSource::WebArchive,
                        "geckocodes.org-201909 archive (recommended)"
                    ),
                    (TxtCodesSource::GameHacking, "gamehacking.org (up to date)"),
                    (TxtCodesSource::Rc24, "codes.rc24.xyz"),
                ],
                state.config.txt_codes_source,
                Message::SetTxtCodesSource
            ),
            setting(
                "Theme",
                Icon::SunMoon,
                ThemePreference::iter().map(|l| (l, l.into())),
                state.config.theme_preference,
                Message::SetThemePreference
            ),
            setting(
                "Preferred language for PAL covers",
                Icon::ImageDown,
                PreferredLanguage::iter().map(|l| (l, l.into())),
                state.config.preferred_language,
                Message::SetPreferredLanguage
            )
        ]
        .spacing(10)
        .padding(padding::left(10).bottom(10).right(20))
        .width(Length::Fill),
    );

    column![titlebar, contents].into()
}
