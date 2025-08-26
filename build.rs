use std::{
    env,
    fs::File,
    io::{BufReader, BufWriter, Write},
    mem::size_of,
    path::Path,
};

use hex::deserialize as deserialize_hex;
use serde::Deserialize;
use zerocopy::{Immutable, IntoBytes, KnownLayout};

// Keep in sync with src/util/redump.rs
#[derive(Clone, Debug, IntoBytes, Immutable, KnownLayout)]
#[repr(C, align(4))]
struct Header {
    entry_count: u32,
    entry_size: u32,
}

// Keep in sync with src/util/redump.rs
#[derive(Clone, Debug, IntoBytes, Immutable, KnownLayout)]
#[repr(C, align(4))]
struct GameEntry {
    crc32: u32,
    string_table_offset: u32,
    md5: [u8; 16],
    sha1: [u8; 20],
}

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
    #[serde(rename = "@crc", deserialize_with = "deserialize_hex")]
    crc32: [u8; 4],
    #[serde(rename = "@md5", deserialize_with = "deserialize_hex")]
    md5: [u8; 16],
    #[serde(rename = "@sha1", deserialize_with = "deserialize_hex")]
    sha1: [u8; 20],
}

fn compile_redump_database() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("parsed-dats.bin");
    let out_file = File::create(dest_path).expect("Failed to open out file");
    let mut out = zstd::Encoder::new(BufWriter::new(out_file), zstd::zstd_safe::max_c_level())
        .expect("Failed to create zstd encoder");

    // Parse dat files
    let mut entries = Vec::<(GameEntry, String)>::new();
    for path in [
        "assets/gc-non-redump.dat",
        "assets/gc-npdp.dat",
        "assets/gc-redump.dat",
        "assets/wii-redump.dat",
    ] {
        println!("cargo:rustc-rerun-if-changed={}", path);
        let file = BufReader::new(File::open(path).expect("Failed to open dat file"));
        let dat: DatFile = quick_xml::de::from_reader(file).expect("Failed to parse dat file");
        entries.extend(dat.games.into_iter().filter_map(|game| {
            if game.roms.len() != 1 {
                return None;
            }
            let rom = &game.roms[0];
            Some((
                GameEntry {
                    string_table_offset: 0,
                    crc32: u32::from_be_bytes(rom.crc32),
                    md5: rom.md5,
                    sha1: rom.sha1,
                },
                game.name,
            ))
        }));
    }

    // Sort by CRC32
    entries.sort_by_key(|(entry, _)| entry.crc32);

    // Calculate total size and store in zstd header
    let entries_size = entries.len() * size_of::<GameEntry>();
    let string_table_size = entries
        .iter()
        .map(|(_, name)| name.len() + 4)
        .sum::<usize>();
    let total_size = size_of::<Header>() + entries_size + string_table_size;
    out.set_pledged_src_size(Some(total_size as u64)).unwrap();
    out.include_contentsize(true).unwrap();

    // Write game entries
    let header = Header {
        entry_count: entries.len() as u32,
        entry_size: size_of::<GameEntry>() as u32,
    };
    out.write_all(header.as_bytes()).unwrap();
    let mut string_table_offset = 0u32;
    for (entry, name) in &mut entries {
        entry.string_table_offset = string_table_offset;
        out.write_all(entry.as_bytes()).unwrap();
        string_table_offset += name.len() as u32 + 4;
    }

    // Write string table
    for (_, name) in &entries {
        out.write_all(&(name.len() as u32).to_le_bytes()).unwrap();
        out.write_all(name.as_bytes()).unwrap();
    }

    // Finalize
    out.finish()
        .expect("Failed to finish zstd encoder")
        .flush()
        .expect("Failed to flush output file");

    println!("cargo:rustc-env=REDUMP_DB_COUNT={}", entries.len());
}

fn main() {
    // Compile Redump database
    compile_redump_database();

    // Windows-specific icon resource
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("logo.ico");
        res.compile().unwrap();
    }
}
