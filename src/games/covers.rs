// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::{Config, PreferredLanguage},
    errors::Error,
    util::http::{download_file, download_file_with_fallback},
};
use smol::stream::{self, Stream, StreamExt};
use std::{
    fmt::Display,
    path::{Path, PathBuf},
};
use wii_disc_info::{RegionCode, game_id::GameID};

#[derive(Debug, Clone, Copy)]
pub enum CoverType {
    Cover3D,
    Cover2D,
    CoverFull,
    Disc,
}

impl CoverType {
    pub fn as_str(&self) -> &'static str {
        match self {
            CoverType::Cover3D => "cover3D",
            CoverType::Cover2D => "cover",
            CoverType::CoverFull => "coverfull",
            CoverType::Disc => "disc",
        }
    }
}

impl Display for CoverType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_str().fmt(f)
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
        RegionCode::NetherlandsEuropeAlternateLanguages => preferred.as_str(),
        RegionCode::Italy => "IT",
        RegionCode::Japan => "JA",
        RegionCode::Korea => "KO",
        RegionCode::JapaneseImportToEuropeAustraliaAndOtherPALRegions => preferred.as_str(),
        RegionCode::AmericanImportToEuropeAustraliaAndOtherPALRegions => preferred.as_str(),
        RegionCode::JapaneseImportToUSAAndOtherNTSCRegions => "US",
        RegionCode::EuropeAndOtherPALRegionsSuchAsAustralia => preferred.as_str(),
        RegionCode::JapaneseVirtualConsoleImportToKorea => "KO",
        RegionCode::Russia => "RU",
        RegionCode::Spain => "ES",
        RegionCode::AmericanVirtualConsoleImportToKorea => "KO",
        RegionCode::AustraliaEuropeAlternateLanguages => preferred.as_str(),
        RegionCode::Scandinavia => preferred.as_str(),
        RegionCode::RepublicOfChinaTaiwanHongKongMacau => "ZH",
        RegionCode::EuropeAlternateLanguagesUSSpecialReleases => preferred.as_str(),
        RegionCode::Unknown => "EN",
    }
}

pub async fn download_cover(
    game_id: GameID,
    cover_type: CoverType,
    dir: &Path,
    preferred_language: PreferredLanguage,
) -> Result<bool, Error> {
    let cover_path = dir.join(game_id.as_str()).with_added_extension("png");
    if cover_path.exists() {
        return Ok(false);
    }

    let lang_str = lang_str(game_id, preferred_language);
    let cover_url = format!("https://art.gametdb.com/wii/{cover_type}/{lang_str}/{game_id}.png");

    if lang_str == "EN" {
        download_file(&cover_url, &cover_path).await?;
    } else {
        let fallback = format!("https://art.gametdb.com/wii/{cover_type}/EN/{game_id}.png");
        download_file_with_fallback(&cover_url, &cover_path, &fallback).await?;
    }

    Ok(true)
}

pub fn download_ui_covers(
    ids: Vec<GameID>,
    data_dir: PathBuf,
    preferred_language: PreferredLanguage,
) -> impl Stream<Item = ()> {
    let covers_dir = data_dir.join("covers");

    stream::iter(ids)
        .then(move |id| {
            let covers_dir = covers_dir.clone();

            async move {
                download_cover(id, CoverType::Cover3D, &covers_dir, preferred_language)
                    .await
                    .unwrap_or(false)
            }
        })
        .filter(|&new| new)
        .map(|_| ())
}

pub async fn download_all_covers_for_usbloadergx(
    ids: impl IntoIterator<Item = GameID> + Copy,
    config: &Config,
) -> Result<Vec<GameID>, Error> {
    let covers_dir = config
        .mount_point
        .join("apps")
        .join("usbloader_gx")
        .join("images");

    let pairs = [
        (covers_dir.clone(), CoverType::Cover3D),
        (covers_dir.join("2D"), CoverType::Cover2D),
        (covers_dir.join("full"), CoverType::CoverFull),
        (covers_dir.join("disc"), CoverType::Disc),
    ];

    let mut failed_ids = Vec::new();
    for (dir, cover_type) in pairs {
        for game_id in ids {
            if download_cover(game_id, cover_type, &dir, config.preferred_language)
                .await
                .is_err()
            {
                failed_ids.push(game_id);
            }
        }
    }

    Ok(failed_ids)
}

pub async fn download_all_covers_for_wiiflow(
    ids: impl IntoIterator<Item = GameID> + Copy,
    config: &Config,
) -> Result<Vec<GameID>, Error> {
    let covers_dir = config.mount_point.join("wiiflow");

    let pairs = [
        ("boxcovers", CoverType::CoverFull),
        ("covers", CoverType::Cover2D),
    ];

    let mut failed_ids = Vec::new();
    for (subdir, cover_type) in pairs {
        let dir = covers_dir.join(subdir);

        for game_id in ids {
            if download_cover(game_id, cover_type, &dir, config.preferred_language)
                .await
                .is_err()
            {
                failed_ids.push(game_id);
            }
        }
    }

    Ok(failed_ids)
}
