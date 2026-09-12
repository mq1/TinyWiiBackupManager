// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::{Config, PreferredLanguage},
    errors::Error,
    util::http::{download_file, download_file_with_fallback},
};
use itertools::Itertools;
use smol::stream::{self, Stream, StreamExt};
use std::{
    convert::identity,
    path::{Path, PathBuf},
};
use strum_macros::Display;
use tap::Pipe;
use wii_disc_info::{RegionCode, game_id::GameID};

#[derive(Debug, Clone, Copy, Display)]
pub enum CoverType {
    #[strum(serialize = "cover3D")]
    Cover3D,

    #[strum(serialize = "cover")]
    Cover2D,

    #[strum(serialize = "coverfull")]
    CoverFull,

    #[strum(serialize = "disc")]
    Disc,
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
    ids: impl IntoIterator<Item = GameID>,
    data_dir: PathBuf,
    preferred_language: PreferredLanguage,
) -> impl Stream<Item = GameID> {
    ids.into_iter()
        .cartesian_product(std::iter::once(data_dir.join("covers")))
        .pipe(stream::iter)
        .then(move |(id, covers_dir)| async move {
            download_cover(id, CoverType::Cover3D, &covers_dir, preferred_language)
                .await
                .unwrap_or(false)
                .then_some(id)
        })
        .filter_map(identity)
}

async fn download_all_covers(
    ids: impl IntoIterator<Item = GameID>,
    pairs: &[(PathBuf, CoverType)],
    config: &Config,
) -> Vec<GameID> {
    ids.into_iter()
        .cartesian_product(pairs)
        .pipe(stream::iter)
        .then(|(game_id, (dir, cover_type))| async move {
            download_cover(game_id, *cover_type, dir, config.preferred_language)
                .await
                .is_err()
                .then_some(game_id)
        })
        .filter_map(identity)
        .collect::<Vec<_>>()
        .await
}

pub async fn download_all_covers_for_usbloadergx(
    ids: impl IntoIterator<Item = GameID>,
    config: &Config,
) -> Vec<GameID> {
    config
        .mount_point
        .join("apps")
        .join("usbloader_gx")
        .join("images")
        .pipe(|covers_dir| {
            [
                (covers_dir.clone(), CoverType::Cover3D),
                (covers_dir.join("2D"), CoverType::Cover2D),
                (covers_dir.join("full"), CoverType::CoverFull),
                (covers_dir.join("disc"), CoverType::Disc),
            ]
        })
        .pipe_ref(move |pairs| download_all_covers(ids, pairs, config))
        .await
}

pub async fn download_all_covers_for_wiiflow(
    ids: impl IntoIterator<Item = GameID>,
    config: &Config,
) -> Vec<GameID> {
    config
        .mount_point
        .join("wiiflow")
        .pipe(|covers_dir| {
            [
                (covers_dir.join("boxcovers"), CoverType::CoverFull),
                (covers_dir.join("covers"), CoverType::Cover2D),
            ]
        })
        .pipe_ref(move |pairs| download_all_covers(ids, pairs, config))
        .await
}
