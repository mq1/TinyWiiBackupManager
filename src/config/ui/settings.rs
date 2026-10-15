// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::{Config, PreferredLanguage, ThemePreference, TxtCodesSource, message::ConfigMessage},
    ui::components::my_group::my_group,
};
use iced::{
    Element, Length, padding,
    widget::{Column, column, radio, scrollable},
};
use lucide_iced::bytes::{
    DISC_3, FILE_MINUS_CORNER, IMAGE_DOWN, SKULL, SQUARE_SPLIT_HORIZONTAL, SUN_MOON, TRASH,
};
use nod::common::Format;
use strum::IntoEnumIterator;

fn setting<T: Eq + Copy>(
    label: &'static str,
    icon_bytes: &'static [u8],
    items: impl IntoIterator<Item = (T, &'static str)>,
    active: T,
    on_change: fn(T) -> ConfigMessage,
) -> Element<'static, ConfigMessage> {
    let content = items
        .into_iter()
        .map(|(value, label)| radio(label, value, Some(active), on_change).into())
        .collect::<Column<'_, _>>()
        .spacing(5);

    my_group(label, content).icon(icon_bytes).into()
}

pub fn settings(config: &Config) -> Element<'_, ConfigMessage> {
    scrollable(
        column![
            setting(
                "Wii output format",
                DISC_3,
                [
                    (Format::Wbfs, "WBFS (Recommended)"),
                    (Format::Iso, "ISO (very large)"),
                ],
                config.wii_output_format(),
                ConfigMessage::SetWiiOutputFormat
            ),
            setting(
                "GameCube output format",
                DISC_3,
                [
                    (Format::Iso, "ISO (Recommended)"),
                    (
                        Format::Ciso,
                        "CISO (much smaller, slightly slower, less metadata in game loaders)"
                    ),
                ],
                config.gc_output_format(),
                ConfigMessage::SetGcOutputFormat
            ),
            setting(
                "Split output",
                SQUARE_SPLIT_HORIZONTAL,
                [
                    (false, "Only when needed (recommended)"),
                    (true, "Always 4GB-32KB"),
                ],
                config.always_split(),
                ConfigMessage::SetAlwaysSplit
            ),
            setting(
                "Remove update partition on WBFS/CISO",
                FILE_MINUS_CORNER,
                [
                    (false, "No (recommended)"),
                    (
                        true,
                        "Yes (saves some space; update partition is zeroed, but still there)"
                    ),
                ],
                config.scrub_update_partition(),
                ConfigMessage::SetScrubUpdatePartition
            ),
            setting(
                "Delete sources when adding games",
                TRASH,
                [(false, "No (recommended)"), (true, "Yes")],
                config.remove_sources_games(),
                ConfigMessage::SetRemoveSourcesGames
            ),
            setting(
                "Delete sources when adding apps",
                TRASH,
                [(false, "No (recommended)"), (true, "Yes")],
                config.remove_sources_apps(),
                ConfigMessage::SetRemoveSourcesApps
            ),
            setting(
                "Cheat codes source",
                SKULL,
                [
                    (
                        TxtCodesSource::WebArchive,
                        "geckocodes.org-201909 archive (recommended)"
                    ),
                    (TxtCodesSource::GameHacking, "gamehacking.org (up to date)"),
                    (TxtCodesSource::Rc24, "codes.rc24.xyz"),
                ],
                config.txt_codes_source(),
                ConfigMessage::SetTxtCodesSource
            ),
            setting(
                "Theme",
                SUN_MOON,
                ThemePreference::iter().map(|l| (l, l.into())),
                config.theme_preference(),
                ConfigMessage::SetThemePreference
            ),
            setting(
                "Preferred language for PAL covers",
                IMAGE_DOWN,
                PreferredLanguage::iter().map(|l| (l, l.into())),
                config.preferred_language(),
                ConfigMessage::SetPreferredLanguage
            )
        ]
        .spacing(10)
        .padding(padding::all(10).right(20))
        .width(Length::Fill),
    )
    .into()
}
