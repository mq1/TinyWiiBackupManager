// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::config::PreferredLanguage;
use crate::game_id::GameID;
use crate::util::AGENT;
use anyhow::{Result, bail};
use arrayvec::ArrayString;
use std::fmt::Write;
use std::{fs, path::Path};
use wii_disc_info::RegionCode;

#[must_use]
fn lang_str(game_id: GameID, preferred: PreferredLanguage) -> &'static str {
    let mut s = ArrayString::<6>::new();
    write!(s, "{game_id}").unwrap();
    let c = s.chars().nth(3).unwrap_or('\0');
    let code = wii_disc_info::RegionCode::from(c);

    match code {
        RegionCode::SystemWiiChannels => "EN",
        RegionCode::UfouriaTheSagaNA => "EN",
        RegionCode::Germany => "DE",
        RegionCode::USA => "EN",
        RegionCode::France => "FR",
        RegionCode::NetherlandsEuropeAlternateLanguages => preferred.as_str(),
        RegionCode::Italy => "IT",
        RegionCode::Japan => "JA",
        RegionCode::Korea => "KO",
        RegionCode::JapaneseImportToEuropeAustraliaAndOtherPALRegions => preferred.as_str(),
        RegionCode::AmericanImportToEuropeAustraliaAndOtherPALRegions => preferred.as_str(),
        RegionCode::JapaneseImportToUSAAndOtherNTSCRegions => "EN",
        RegionCode::EuropeAndOtherPALRegionsSuchAsAustralia => preferred.as_str(),
        RegionCode::JapaneseVirtualConsoleImportToKorea => "KO",
        RegionCode::Russia => "RU",
        RegionCode::Spain => "ES",
        RegionCode::AmericanVirtualConsoleImportToKorea => "KO",
        RegionCode::AustraliaEuropeAlternateLanguages => preferred.as_str(),
        RegionCode::Scandinavia => preferred.as_str(),
        RegionCode::RepublicOfChinaTaiwanHongKongMacau => "ZH",
        RegionCode::EuropeAlternateLanguagesUSSpecialReleases => preferred.as_str(),
        RegionCode::Unknown(_) => "EN",
    }
}

pub fn download_cover(
    game_id: GameID,
    covers_dir: &Path,
    preferred_language: PreferredLanguage,
) -> Result<()> {
    let mut filename = ArrayString::<10>::new();
    write!(&mut filename, "{game_id}.png")?;

    let cover_path = covers_dir.join(filename);

    if cover_path.exists() {
        bail!("Cover already exists");
    }

    let lang_str = lang_str(game_id, preferred_language);
    let cover_url = format!("https://art.gametdb.com/wii/cover3D/{lang_str}/{game_id}.png");

    fn get(url: &str) -> Result<Vec<u8>, ureq::Error> {
        AGENT.get(url).call()?.body_mut().read_to_vec()
    }

    let body = match get(&cover_url) {
        Ok(body) => body,
        Err(_) => {
            let url = format!("https://art.gametdb.com/wii/cover3D/EN/{game_id}.png");
            get(&url)?
        }
    };

    fs::write(&cover_path, &body)?;

    Ok(())
}
