use std::{str, sync::OnceLock};
use zerocopy::{FromBytes, FromZeros, Immutable, IntoBytes, KnownLayout};

#[derive(Clone, Debug)]
pub struct GameResult {
    pub name: String,
    pub crc32: u32,
    pub md5: [u8; 16],
    pub sha1: [u8; 20],
}

impl GameResult {
    /// Check if this entry matches the provided hashes
    pub fn matches(&self, crc32: Option<u32>, sha1: Option<[u8; 20]>) -> bool {
        if let Some(crc) = crc32
            && self.crc32 != crc
        {
            return false;
        }
        if let Some(sha) = sha1
            && self.sha1 != sha
        {
            return false;
        }
        true
    }

    /// Check if all provided hashes match (for full verification)
    pub fn full_match(&self, crc32: u32, sha1: [u8; 20]) -> bool {
        self.crc32 == crc32 && self.sha1 == sha1
    }
}

pub fn find_by_crc32(crc32: u32) -> Option<GameResult> {
    let data = loaded_data();
    let (header, remaining) = Header::ref_from_prefix(data).ok()?;
    assert_eq!(header.entry_size as usize, size_of::<GameEntry>());

    let entries_size = header.entry_count as usize * size_of::<GameEntry>();
    let (entries_buf, string_table) = remaining.split_at(entries_size);
    let entries = <[GameEntry]>::ref_from_bytes(entries_buf).ok()?;

    // Binary search by CRC32
    let index = entries
        .binary_search_by_key(&crc32, |entry| entry.crc32)
        .ok()?;

    // Parse the entry
    let entry = &entries[index];
    let offset = entry.string_table_offset as usize;
    let name_size = u32::from_le_bytes([
        string_table[offset],
        string_table[offset + 1],
        string_table[offset + 2],
        string_table[offset + 3],
    ]) as usize;
    let name = str::from_utf8(&string_table[offset + 4..offset + 4 + name_size])
        .unwrap()
        .to_string();

    Some(GameResult {
        name,
        crc32: entry.crc32,
        md5: entry.md5,
        sha1: entry.sha1,
    })
}

const BUILTIN: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/parsed-dats.bin"));
static LOADED: OnceLock<Box<[u8]>> = OnceLock::new();

fn loaded_data() -> &'static [u8] {
    LOADED
        .get_or_init(|| {
            let size = zstd::zstd_safe::get_frame_content_size(BUILTIN)
                .unwrap()
                .unwrap() as usize;
            let mut out = <[u8]>::new_box_zeroed_with_elems(size).unwrap();
            let out_size = zstd::bulk::Decompressor::new()
                .unwrap()
                .decompress_to_buffer(BUILTIN, out.as_mut())
                .unwrap();
            debug_assert_eq!(out_size, size);
            out
        })
        .as_ref()
}

// Keep in sync with build.rs
#[derive(Clone, Debug, IntoBytes, FromBytes, Immutable, KnownLayout)]
#[repr(C, align(4))]
struct Header {
    entry_count: u32,
    entry_size: u32,
}

// Keep in sync with build.rs
#[derive(Clone, Debug, IntoBytes, FromBytes, Immutable, KnownLayout)]
#[repr(C, align(4))]
struct GameEntry {
    crc32: u32,
    string_table_offset: u32,
    md5: [u8; 16],
    sha1: [u8; 20],
}
