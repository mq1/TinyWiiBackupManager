// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

#![allow(dead_code)]

use rkyv::{Archive, Serialize, rancor};
use serde::Deserialize;
use std::{
    env,
    fs::{self, File},
    io::BufReader,
    path::{Path, PathBuf},
};

// Top-level root element <datafile>
#[derive(Debug, Deserialize)]
struct Datafile {
    #[serde(rename = "game")]
    pub games: Vec<Game>,
}

// <game> element
#[derive(Debug, Deserialize)]
struct Game {
    #[serde(rename = "@name")]
    pub name: String,

    pub id: String, // NMTOKEN
    pub r#type: String,
    pub region: String,
    pub languages: String,

    #[serde(rename = "locale", default)] // default handles zeroOrMore
    pub locales: Vec<Locale>,

    pub developer: Option<String>,
    pub publisher: String,
    pub date: Date,
    pub genre: Option<String>,
    pub rating: Rating,

    #[serde(rename = "wi-fi")]
    pub wifi: Wifi,

    pub input: Input,
    pub save: Option<Save>,

    #[serde(rename = "rom")]
    pub roms: Vec<Rom>,

    pub case: Option<Case>,
}

// <locale> element
#[derive(Debug, Deserialize)]
struct Locale {
    #[serde(rename = "@lang")]
    pub lang: String, // NCName
    pub title: String,
    pub synopsis: String,
}

// <date> element
#[derive(Debug, Deserialize)]
struct Date {
    #[serde(rename = "@day")]
    pub day: String,
    #[serde(rename = "@month")]
    pub month: String,
    #[serde(rename = "@year")]
    pub year: String,
}

// <rating> element
#[derive(Debug, Deserialize)]
struct Rating {
    #[serde(rename = "@type")]
    pub r#type: String,
    #[serde(rename = "@value")]
    pub value: String,
    #[serde(rename = "descriptor", default)] // default handles zeroOrMore
    pub descriptors: Vec<String>,
}

// <wi-fi> element
#[derive(Debug, Deserialize)]
struct Wifi {
    #[serde(rename = "@players")]
    pub players: i64,
    #[serde(rename = "feature", default)] // default handles zeroOrMore
    pub features: Vec<String>, // NCName
}

// <input> element
#[derive(Debug, Deserialize)]
struct Input {
    #[serde(rename = "@players")]
    pub players: i64,
    #[serde(rename = "control", default)] // default handles zeroOrMore
    pub controls: Vec<Control>,
}

// <control> element
#[derive(Debug, Deserialize)]
struct Control {
    #[serde(rename = "@required")]
    pub required: bool,
    #[serde(rename = "@type")]
    pub r#type: String,
}

// <save> element
#[derive(Debug, Deserialize)]
struct Save {
    #[serde(rename = "@blocks")]
    pub blocks: i64,
    #[serde(rename = "@copy")]
    pub copy: Option<bool>,
    #[serde(rename = "@move")]
    pub r#move: Option<bool>,
}

// <rom> element
#[derive(Debug, Deserialize)]
struct Rom {
    #[serde(rename = "@crc")]
    pub crc: Option<String>, // NMTOKEN
    #[serde(rename = "@md5")]
    pub md5: Option<String>, // NMTOKEN
    #[serde(rename = "@name")]
    pub name: Option<String>,
    #[serde(rename = "@sha1")]
    pub sha1: Option<String>,
    #[serde(rename = "@size")]
    pub size: Option<i64>,
    #[serde(rename = "@version")]
    pub version: String,
}

// <case> element
#[derive(Debug, Deserialize)]
struct Case {
    #[serde(rename = "@color")]
    pub color: Option<String>,
    #[serde(rename = "@versions")]
    pub versions: Option<i64>,
}

#[rustfmt::skip]
#[derive(Serialize, Deserialize, Archive)]
#[serde(rename_all(deserialize = "UPPERCASE"))]
enum Language { En, Fr, De, Es, It, Ja, Nl, Se, Dk, No, Ko, Pt, Zhtw, Zhcn, Fi, Tr, Gr, Ru }

#[rustfmt::skip]
#[derive(Serialize, Deserialize, Archive)]
#[serde(rename_all(deserialize = "SCREAMING-KEBAB-CASE"))]
enum Region { NtscJ, NtscU, NtscK, NtscT, Pal, PalR }

#[rustfmt::skip]
#[derive(Serialize, Archive)]
struct GameInfo {
    id: [u8; 6],
    name: String,
    region: Region,
    languages: Vec<Language>,
    crc_list: Vec<u32>,
}

fn compile_wiitdb_xml() {
    let xml = BufReader::new(File::open("assets/wiitdb.xml").expect("Failed to open wiitdb.xml"));
    let data: Datafile = quick_xml::de::from_reader(xml).expect("Failed to parse wiitdb.xml");

    let mut entries = Vec::new();
    for game in data.games {
        let mut id = [0u8; 6];
        let bytes = game.id.as_bytes();
        let len = bytes.len().min(6);
        id[..len].copy_from_slice(&bytes[..len]);

        let name = game.name.trim().to_string();

        // skip invalid games
        if id.is_empty()
            || game.region.is_empty()
            || game.languages.is_empty()
            || name.is_empty()
            || game.roms.is_empty()
        {
            continue;
        }

        let region = serde_plain::from_str::<Region>(&game.region).unwrap();

        let languages = game
            .languages
            .split(',')
            .map(|s| serde_plain::from_str::<Language>(s).unwrap())
            .collect::<Vec<_>>();

        let crc_list = game
            .roms
            .iter()
            .filter_map(|r| r.crc.clone())
            .filter_map(|crc| u32::from_str_radix(&crc, 16).ok())
            .collect::<Vec<_>>();

        entries.push(GameInfo {
            id,
            name,
            region,
            languages,
            crc_list,
        });
    }

    // Sort by ID to enable binary search
    entries.sort_by_key(|game| game.id);

    let serialized = rkyv::to_bytes::<rancor::Error>(&entries).expect("Rkyv serialization failed");
    let compressed = zstd::bulk::compress(&serialized, 19).expect("Zstd compression failed");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let dest_path = out_dir.join("wiitdb.bin.zst");
    fs::write(&dest_path, compressed).expect("Failed to write compressed data");

    let metadata = format!("const WIITDB_SIZE: usize = {};", serialized.len());
    let metadata_path = Path::new(&out_dir).join("metadata.rs");
    fs::write(&metadata_path, metadata).unwrap();
}

fn main() {
    // Re-run the build script if wiitdb.xml changes
    println!("cargo:rerun-if-changed=assets/wiitdb.xml");

    // Compile wiitdb.xml
    compile_wiitdb_xml();

    // Windows-specific icon resource
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/logo.ico");
        res.compile().unwrap();
    }
}
