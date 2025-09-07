// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

#![allow(dead_code)]

use heck::ToUpperCamelCase;
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
        let mut id = [0u8; 6];
        let id_bytes = game.id.as_bytes();
        id[..id_bytes.len()].copy_from_slice(&id_bytes);

        let name = format!("\"{}\"", game.name);

        // if region not found, skip game
        if game.region.is_empty() {
            continue;
        }

        let region = format!("Region::{}", game.region.to_upper_camel_case());

        // if language is empty, skip game
        if game.languages.is_empty() {
            continue;
        }

        // build languages list string
        let languages = format!(
            "&[{}]",
            game.languages
                .split(',')
                .map(|lang| format!("Language::{}", lang))
                .collect::<Vec<_>>()
                .join(",")
        );

        // Parse CRCs. Invalid CRCs are skipped.
        let crc_list = format!(
            "&[{}]",
            game.roms
                .into_iter()
                .filter_map(|rom| rom.crc)
                .filter_map(|crc| u32::from_str_radix(&crc, 16).ok())
                .map(|crc| crc.to_string())
                .collect::<Vec<_>>()
                .join(",")
        );

        map_builder.entry(
            id,
            format!(
                "&GameInfo {{ name: {name}, region: {region}, languages: {languages}, crc_list: {crc_list} }}",
            ),
        );
    }

    // Write the generated map directly into the output file.
    // This is not just a variable, but the full phf::phf_map! macro invocation.
    writeln!(
        &mut file,
        "static GAMES: phf::Map<[u8; 6], &'static GameInfo> = {};",
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
