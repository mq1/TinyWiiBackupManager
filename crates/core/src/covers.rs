// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::config::{Config, PreferredLanguage};
use crate::game_id::GameID;
use crate::util::AGENT;
use anyhow::Result;
use derive_more::Display;
use std::{fs, io::Write, path::Path};
use wii_disc_info::RegionCode;

#[derive(Debug, Clone, Copy, Display)]
pub enum CoverType {
    #[display("cover3D")]
    Cover3D,

    #[display("cover")]
    Cover2D,

    #[display("coverfull")]
    CoverFull,

    #[display("disc")]
    Disc,
}

#[must_use]
fn lang_str(game_id: GameID, preferred: PreferredLanguage) -> &'static str {
    let mut buf = [0u8; 6];
    write!(&mut buf[..], "{game_id}").unwrap();
    let char_byte = buf[3];

    let code = wii_disc_info::RegionCode::from(char_byte);

    match code {
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

pub fn download_cover(
    game_id: GameID,
    cover_type: CoverType,
    dir: &Path,
    preferred_language: PreferredLanguage,
) -> Result<bool> {
    let filename = format!("{game_id}.png");
    let cover_path = dir.join(&filename);

    if cover_path.exists() {
        return Ok(false);
    }

    let lang_str = lang_str(game_id, preferred_language);
    let cover_url = format!("https://art.gametdb.com/wii/{cover_type}/{lang_str}/{game_id}.png");

    fn get(url: &str) -> Result<Vec<u8>, ureq::Error> {
        AGENT.get(url).call()?.body_mut().read_to_vec()
    }

    let body = match get(&cover_url) {
        Ok(body) => body,
        Err(_) if lang_str != "EN" => {
            let url = format!("https://art.gametdb.com/wii/{cover_type}/EN/{game_id}.png");
            get(&url)?
        }
        Err(e) => return Err(e.into()),
    };

    fs::write(&cover_path, body)?;

    Ok(true)
}

pub fn download_all_covers_for_usbloadergx(ids: &[GameID], config: &Config) -> Result<Vec<GameID>> {
    let covers_dir = config
        .contents
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
        fs::create_dir_all(&dir)?;

        for game_id in ids {
            if download_cover(
                *game_id,
                cover_type,
                &dir,
                config.contents.preferred_language,
            )
            .is_err()
            {
                failed_ids.push(*game_id);
            }
        }
    }

    Ok(failed_ids)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ConfigContents;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn usbloadergx_download_creates_expected_cover_directories() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let mount = std::env::temp_dir().join(format!("twbm-covers-{nanos}"));
        let config = Config {
            path: mount.join("config.json"),
            contents: ConfigContents {
                mount_point: mount.clone(),
                ..Default::default()
            },
        };

        download_all_covers_for_usbloadergx(&[], &config).unwrap();

        let covers_dir = mount.join("apps").join("usbloader_gx").join("images");
        assert!(covers_dir.is_dir());
        assert!(covers_dir.join("2D").is_dir());
        assert!(covers_dir.join("full").is_dir());
        assert!(covers_dir.join("disc").is_dir());

        fs::remove_dir_all(mount).unwrap();
    }
}

pub fn download_all_covers_for_wiiflow(ids: &[GameID], config: &Config) -> Result<Vec<GameID>> {
    let covers_dir = config.contents.mount_point.join("wiiflow");

    let pairs = [
        ("boxcovers", CoverType::CoverFull),
        ("covers", CoverType::Cover2D),
    ];

    let mut failed_ids = Vec::new();
    for (subdir, cover_type) in pairs {
        let dir = covers_dir.join(subdir);
        fs::create_dir_all(&dir)?;

        for game_id in ids {
            if download_cover(
                *game_id,
                cover_type,
                &dir,
                config.contents.preferred_language,
            )
            .is_err()
            {
                failed_ids.push(*game_id);
            }
        }
    }

    Ok(failed_ids)
}
