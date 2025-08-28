// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

#![allow(dead_code)]

use serde::Deserialize;
use std::{
    env,
    fs::File,
    io::{BufReader, BufWriter, Write},
    path::Path,
};

// Top-level root element <datafile>
#[derive(Debug, Deserialize)]
struct Datafile {
    #[serde(rename = "game")]
    pub games: Vec<Game>,
}

// <WiiTDB> element
#[derive(Debug, Deserialize)]
struct WiiTdb {
    #[serde(rename = "@games")]
    pub games: i64,
    #[serde(rename = "@version")]
    pub version: i64,
}

// <companies> element
#[derive(Debug, Deserialize)]
struct Companies {
    #[serde(rename = "company")]
    pub company_list: Vec<Company>,
}

// <company> element
#[derive(Debug, Deserialize)]
struct Company {
    #[serde(rename = "@code")]
    pub code: String, // NMTOKEN
    #[serde(rename = "@name")]
    pub name: String,
}

// <genres> element
#[derive(Debug, Deserialize)]
struct Genres {
    #[serde(rename = "maingenre")]
    pub main_genre_list: Vec<MainGenre>,
}

// <maingenre> element
#[derive(Debug, Deserialize)]
struct MainGenre {
    #[serde(rename = "@name")]
    pub name: String, // NCName
    #[serde(rename = "loc")]
    pub locs: Vec<Loc>,
    #[serde(rename = "subgenre")]
    pub sub_genres: Vec<SubGenre>,
}

// <subgenre> element
#[derive(Debug, Deserialize)]
struct SubGenre {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "loc")]
    pub locs: Vec<Loc>,
}

// <descriptors> element
#[derive(Debug, Deserialize)]
struct Descriptors {
    #[serde(rename = "descr")]
    pub descr_list: Vec<Descr>,
}

// <descr> element
#[derive(Debug, Deserialize)]
struct Descr {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "loc")]
    pub locs: Vec<Loc>,
}

// Reusable <loc> element from <define name="loc">
#[derive(Debug, Deserialize)]
struct Loc {
    #[serde(rename = "@lang")]
    pub lang: String, // NCName
    #[serde(rename = "$text")]
    pub text: String,
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

/// Converts a string slice (up to 8 chars) into a u64.
///
/// It effectively treats the string's bytes as a big-endian integer.
/// For example, "ABCD" becomes 0x41424344.
fn game_id_to_u64(id: &str) -> u64 {
    id.bytes().fold(0, |acc, byte| (acc << 8) | u64::from(byte))
}

fn compile_wiitdb_xml() {
    // Path for the generated code snippet in the build output directory
    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("wiitdb_data.rs");
    let mut file = BufWriter::new(File::create(&path).unwrap());

    // Re-run the build script if wiitdb.xml changes
    println!("cargo:rerun-if-changed=assets/wiitdb.xml");

    let xml = BufReader::new(File::open("assets/wiitdb.xml").expect("Failed to open wiitdb.xml"));
    let data: Datafile = quick_xml::de::from_reader(xml).expect("Failed to parse wiitdb.xml");

    let mut map_builder = phf_codegen::Map::new();
    for game in data.games {
        let id = game_id_to_u64(&game.id);

        // replace , with ","
        let languages = game.languages.replace(",", "\",\"");

        map_builder.entry(
            id,
            format!(
                "&GameInfo {{ name: r#\"{}\"#, region: \"{}\", languages: &[\"{}\"] }}",
                game.name, game.region, languages
            ),
        );
    }

    // Write the generated map directly into the output file.
    // This is not just a variable, but the full phf::phf_map! macro invocation.
    writeln!(
        &mut file,
        "static GAMES: phf::Map<u64, &'static GameInfo> = {};",
        map_builder.build()
    )
    .unwrap();
}

fn main() {
    // Compile wiitdb.xml
    compile_wiitdb_xml();

    // Windows-specific icon resource
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("logo.ico");
        res.compile().unwrap();
    }
}
