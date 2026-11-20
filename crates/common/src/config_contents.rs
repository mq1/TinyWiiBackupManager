// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use serde::{Deserialize, Serialize};
use std::{collections::HashSet, path::PathBuf};
use strum_macros::{EnumIter, IntoStaticStr};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ConfigContents {
    pub always_split: bool,
    pub mount_point: Option<PathBuf>,
    pub remove_sources_apps: bool,
    pub remove_sources_games: bool,
    pub scrub_update_partition: bool,
    pub sort_by: SortBy,
    pub view_as: ViewAs,
    pub show_wii: bool,
    pub show_gc: bool,
    pub wii_ip: String,
    pub txt_codes_source: TxtCodesSource,
    pub theme_preference: ThemePreference,
    pub wii_output_format: String,
    pub gc_output_format: String,
    pub known_drives: HashSet<PathBuf>,
    pub preferred_language: PreferredLanguage,
}

impl Default for ConfigContents {
    fn default() -> Self {
        Self {
            always_split: false,
            mount_point: None,
            remove_sources_apps: false,
            remove_sources_games: false,
            scrub_update_partition: false,
            sort_by: SortBy::default(),
            view_as: ViewAs::default(),
            wii_ip: "192.168.1.100".into(),
            txt_codes_source: TxtCodesSource::default(),
            theme_preference: ThemePreference::default(),
            wii_output_format: "wbfs".into(),
            gc_output_format: "iso".into(),
            show_wii: true,
            show_gc: true,
            known_drives: HashSet::new(),
            preferred_language: system_language(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ViewAs {
    #[default]
    Grid,
    Table,
}

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default, EnumIter, IntoStaticStr,
)]
#[serde(rename_all = "lowercase")]
pub enum ThemePreference {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum TxtCodesSource {
    #[default]
    WebArchive,
    GameHacking,
    Rc24,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, EnumIter, IntoStaticStr)]
#[serde(rename_all = "lowercase")]
pub enum PreferredLanguage {
    English,
    French,
    German,
    Spanish,
    Italian,
    Dutch,
    Portuguese,
    Swedish,
    Danish,
    Finnish,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum SortBy {
    #[default]
    NameDescending,
    NameAscending,
    SizeDescending,
    SizeAscending,
}

impl PreferredLanguage {
    pub fn into_str(self) -> &'static str {
        match self {
            PreferredLanguage::English => "EN",
            PreferredLanguage::French => "FR",
            PreferredLanguage::German => "DE",
            PreferredLanguage::Spanish => "ES",
            PreferredLanguage::Italian => "IT",
            PreferredLanguage::Dutch => "NL",
            PreferredLanguage::Portuguese => "PT",
            PreferredLanguage::Swedish => "SW",
            PreferredLanguage::Danish => "DK",
            PreferredLanguage::Finnish => "FI",
        }
    }
}

fn system_language() -> PreferredLanguage {
    let locale = sys_locale::get_locale();

    match locale.as_ref().and_then(|l| l.get(..2)) {
        Some("fr") => PreferredLanguage::French,
        Some("de") => PreferredLanguage::German,
        Some("es") => PreferredLanguage::Spanish,
        Some("it") => PreferredLanguage::Italian,
        Some("nl") => PreferredLanguage::Dutch,
        Some("pt") => PreferredLanguage::Portuguese,
        Some("sv") => PreferredLanguage::Swedish,
        Some("da") => PreferredLanguage::Danish,
        Some("fi") => PreferredLanguage::Finnish,
        _ => PreferredLanguage::English,
    }
}
