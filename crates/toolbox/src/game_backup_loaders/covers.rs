// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;
use std::{
    fmt,
    path::{Path, PathBuf},
};
use twbm_common::config_contents::{ConfigContents, PreferredLanguage};
use twbm_http::{download_file, download_file_with_fallback};
use wii_disc_info::{RegionCode, game_id::GameID};

#[derive(Debug, Clone, Copy)]
pub enum CoverType {
    Cover3D,
    Cover2D,
    CoverFull,
    Disc,
}

impl CoverType {
    fn into_str(self) -> &'static str {
        match self {
            CoverType::Cover3D => "cover3D",
            CoverType::Cover2D => "cover",
            CoverType::CoverFull => "coverfull",
            CoverType::Disc => "disc",
        }
    }
}

impl fmt::Display for CoverType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.into_str().fmt(f)
    }
}

#[must_use]
fn lang_str(game_id: GameID, preferred: PreferredLanguage) -> &'static str {
    match game_id.region() {
        RegionCode::SystemWiiChannels => "EN",
        RegionCode::UfouriaTheSagaNA => "EN",
        RegionCode::Germany => "DE",
        RegionCode::USA => "US",
        RegionCode::France => "FR",
        RegionCode::NetherlandsEuropeAlternateLanguages => preferred.into_str(),
        RegionCode::Italy => "IT",
        RegionCode::Japan => "JA",
        RegionCode::Korea => "KO",
        RegionCode::JapaneseImportToEuropeAustraliaAndOtherPALRegions => preferred.into_str(),
        RegionCode::AmericanImportToEuropeAustraliaAndOtherPALRegions => preferred.into_str(),
        RegionCode::JapaneseImportToUSAAndOtherNTSCRegions => "US",
        RegionCode::EuropeAndOtherPALRegionsSuchAsAustralia => preferred.into_str(),
        RegionCode::JapaneseVirtualConsoleImportToKorea => "KO",
        RegionCode::Russia => "RU",
        RegionCode::Spain => "ES",
        RegionCode::AmericanVirtualConsoleImportToKorea => "KO",
        RegionCode::AustraliaEuropeAlternateLanguages => preferred.into_str(),
        RegionCode::Scandinavia => preferred.into_str(),
        RegionCode::RepublicOfChinaTaiwanHongKongMacau => "ZH",
        RegionCode::EuropeAlternateLanguagesUSSpecialReleases => preferred.into_str(),
        RegionCode::Unknown => "EN",
    }
}

fn download_cover(
    game_id: GameID,
    cover_type: CoverType,
    dir: &Path,
    preferred_language: PreferredLanguage,
) -> Result<bool> {
    let cover_path = dir.join(game_id.as_str()).with_added_extension("png");
    if cover_path.exists() {
        return Ok(false);
    }

    let lang_str = lang_str(game_id, preferred_language);
    let cover_url = format!("https://art.gametdb.com/wii/{cover_type}/{lang_str}/{game_id}.png");

    if lang_str == "EN" {
        download_file(&cover_url, &cover_path)?;
    } else {
        let fallback = format!("https://art.gametdb.com/wii/{cover_type}/EN/{game_id}.png");
        download_file_with_fallback(&cover_url, &cover_path, &fallback)?;
    }

    Ok(true)
}

pub fn download_ui_covers(
    game_ids: Box<[GameID]>,
    data_dir: &'static Path,
    preferred_language: PreferredLanguage,
) -> impl Iterator<Item = GameID> {
    let covers_dir = data_dir.join("covers");

    game_ids.into_iter().filter_map(move |game_id| {
        download_cover(game_id, CoverType::Cover3D, &covers_dir, preferred_language)
            .ok()
            .map(|_| game_id)
    })
}

fn download_all_covers(
    game_ids: Box<[GameID]>,
    pairs: &[(PathBuf, CoverType)],
    preferred_language: PreferredLanguage,
) -> Vec<GameID> {
    let mut errored = Vec::new();

    for game_id in game_ids {
        for (dir, cover_type) in pairs {
            if download_cover(game_id, *cover_type, dir, preferred_language).is_err() {
                errored.push(game_id);
            }
        }
    }

    errored
}

pub fn download_all_covers_for_usbloadergx(
    game_ids: Box<[GameID]>,
    config: &ConfigContents,
) -> Vec<GameID> {
    let Some(mount_point) = config.mount_point.as_deref() else {
        return Vec::new();
    };

    let covers_dir = mount_point.join("apps").join("usbloader_gx").join("images");

    let pairs = [
        (covers_dir.clone(), CoverType::Cover3D),
        (covers_dir.join("2D"), CoverType::Cover2D),
        (covers_dir.join("full"), CoverType::CoverFull),
        (covers_dir.join("disc"), CoverType::Disc),
    ];

    download_all_covers(game_ids, &pairs, config.preferred_language)
}

pub fn download_all_covers_for_wiiflow(
    game_ids: Box<[GameID]>,
    config: &ConfigContents,
) -> Vec<GameID> {
    let Some(mount_point) = config.mount_point.as_deref() else {
        return Vec::new();
    };

    let covers_dir = mount_point.join("wiiflow");

    let pairs = [
        (covers_dir.join("boxcovers"), CoverType::CoverFull),
        (covers_dir.join("covers"), CoverType::Cover2D),
    ];

    download_all_covers(game_ids, &pairs, config.preferred_language)
}
