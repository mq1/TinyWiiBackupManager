// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::config::{PreferredLanguage, SortBy, ThemePreference, TxtCodesSource, ViewAs};
use nod::common::Format;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum ConfigMessage {
    SetMountPoint(PathBuf),
    SetWiiIp(String),
    SetShowWii(bool),
    SetShowGc(bool),
    SetSortBy(SortBy),
    SetViewAs(ViewAs),
    SetAlwaysSplit(bool),
    SetRemoveSourcesGames(bool),
    SetRemoveSourcesApps(bool),
    SetScrubUpdatePartition(bool),
    SetTxtCodesSource(TxtCodesSource),
    SetThemePreference(ThemePreference),
    SetWiiOutputFormat(Format),
    SetGcOutputFormat(Format),
    SetPreferredLanguage(PreferredLanguage),
    NewMountPoint,
    Write,
    Written(Result<(), String>),
}
