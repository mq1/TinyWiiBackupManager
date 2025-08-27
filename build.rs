use serde::Deserialize;
use std::{
    collections::HashMap,
    env,
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    path::Path,
};

fn compile_redump_database() {
    #[derive(Clone, Debug, Deserialize)]
    struct DatFile {
        #[serde(rename = "game")]
        games: Vec<DatGame>,
    }

    #[derive(Clone, Debug, Deserialize)]
    struct DatGame {
        #[serde(rename = "@name")]
        name: String,
        #[serde(rename = "rom")]
        roms: Vec<DatGameRom>,
    }

    #[derive(Clone, Debug, Deserialize)]
    struct DatGameRom {
        #[serde(rename = "@size")]
        #[allow(dead_code)]
        size: u64,
        #[serde(rename = "@crc")]
        crc32: String,
        #[serde(rename = "@md5")]
        md5: String,
        #[serde(rename = "@sha1")]
        sha1: String,
    }

    // Path for the generated code snippet in the build output directory
    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("redump.rs");
    let mut file = BufWriter::new(File::create(&path).unwrap());

    // Re-run the build script if any of the dat files change
    for path in [
        "assets/gc-non-redump.dat",
        "assets/gc-npdp.dat",
        "assets/gc-redump.dat",
        "assets/wii-redump.dat",
    ] {
        println!("cargo:rustc-rerun-if-changed={}", path);
    }

    // Use a HashMap to handle duplicates (last entry wins)
    let mut entries = HashMap::new();

    // Parse dat files and collect all entries
    for path in [
        "assets/gc-non-redump.dat",
        "assets/gc-npdp.dat",
        "assets/gc-redump.dat",
        "assets/wii-redump.dat",
    ] {
        let file = BufReader::new(File::open(path).expect("Failed to open dat file"));
        let dat: DatFile = quick_xml::de::from_reader(file).expect("Failed to parse dat file");

        for game in dat.games {
            if game.roms.len() != 1 {
                continue;
            }
            let rom = &game.roms[0];

            // Format MD5 hex string directly into byte array format
            let md5_bytes_str = rom
                .md5
                .as_bytes()
                .chunks(2)
                .map(|chunk| format!("0x{}{}", chunk[0] as char, chunk[1] as char))
                .collect::<Vec<_>>()
                .join(", ");

            // Format SHA1 hex string directly into byte array format
            let sha1_bytes_str = rom
                .sha1
                .as_bytes()
                .chunks(2)
                .map(|chunk| format!("0x{}{}", chunk[0] as char, chunk[1] as char))
                .collect::<Vec<_>>()
                .join(", ");

            // Parse CRC32 hex string into u32
            let key = u32::from_str_radix(&rom.crc32, 16).expect("Failed to parse CRC32 as hex");

            let value = format!(
                "GameResult {{ name: r#\"{}\"#, crc32: {}, md5: [{}], sha1: [{}] }}",
                game.name, key, md5_bytes_str, sha1_bytes_str
            );

            // Insert or update the entry (later files will override earlier ones)
            entries.insert(key, value);
        }
    }

    // Build the PHF map from the deduplicated entries
    let mut map_builder = phf_codegen::Map::new();
    for (crc32, value) in entries {
        map_builder.entry(crc32, value);
    }

    // Write the generated map directly into the output file.
    // This is not just a variable, but the full phf::phf_map! macro invocation.
    writeln!(
        &mut file,
        "static REDUMP_DB: phf::Map<u32, GameResult> = {};",
        map_builder.build()
    )
    .unwrap();
}

fn compile_titles() {
    // Path for the generated code snippet in the build output directory
    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("titles.rs");
    let mut file = BufWriter::new(File::create(&path).unwrap());

    // Re-run the build script if titles.txt changes
    println!("cargo:rerun-if-changed=assets/titles.txt");

    let titles_file = File::open("assets/titles.txt").expect("Could not open assets/titles.txt");
    let reader = BufReader::new(titles_file);

    let mut map_builder = phf_codegen::Map::new();

    for line in reader.lines() {
        let line = line.unwrap();
        if let Some((game_id, game_title)) = line.split_once('=') {
            let game_id = game_id.trim().to_string();
            let game_title = game_title.trim().to_string();

            map_builder.entry(game_id, format!("r#\"{}\"#", game_title));
        }
    }

    // Write the generated map directly into the output file.
    // This is not just a variable, but the full phf::phf_map! macro invocation.
    writeln!(
        &mut file,
        "static GAME_TITLES: phf::Map<&'static str, &'static str> = {};",
        map_builder.build()
    )
    .unwrap();
}

fn main() {
    // Compile Redump database
    compile_redump_database();

    // Compile titles map
    compile_titles();

    // Windows-specific icon resource
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("logo.ico");
        res.compile().unwrap();
    }
}
