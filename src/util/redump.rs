use serde::{Deserialize, Serialize};

/// Game result from Redump database lookup
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameResult {
    pub name: &'static str,
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

// Include the generated PHF map
include!(concat!(env!("OUT_DIR"), "/redump.rs"));

/// Find a game by its CRC32 checksum
pub fn find_by_crc32(crc32: u32) -> Option<GameResult> {
    REDUMP_DB.get(&crc32).cloned()
}

/// Get the path to a game's redump page
pub fn get_redump_url(crc32: u32) -> Option<String> {
    find_by_crc32(crc32).map(|_| format!("http://redump.org/disc/{}", crc32))
}
