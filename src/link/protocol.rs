// SPDX-FileCopyrightText: 2026 Yanik Fuchs <yanik.fuchs@me.com>
// SPDX-License-Identifier: GPL-3.0-only

//! The wire format of the Link protocol, see docs/LINK_PROTOCOL.md

use std::{
    collections::HashMap,
    fmt,
    io::{self, Read, Write},
};

pub const PORT: u16 = 34013;
pub const QUERY: &[u8] = b"ugbm?";
pub const LEGACY_QUERY: &[u8] = b"vcinject?";
pub const MAX_PAYLOAD: u32 = 8 * 1024 * 1024;
pub const PROTOCOL_VERSION: u32 = 1;

pub mod command {
    pub const INFO: u32 = 1;
    pub const TITLES: u32 = 2;
    pub const METADATA: u32 = 3;
    pub const READ_FILE: u32 = 4;
    pub const WRITE_FILE: u32 = 5;
    pub const LIST_DIR: u32 = 6;
    pub const INSTALL_LEGACY: u32 = 7;
    pub const DELETE: u32 = 8;
    pub const REBOOT: u32 = 9;
    pub const REPORT: u32 = 10;
    pub const NOTE: u32 = 11;
    pub const METADATA_BATCH: u32 = 12;
    pub const PROGRESS: u32 = 13;
    pub const RESTART: u32 = 14;
    pub const LAUNCH: u32 = 18;
    pub const INSTALL_STREAM: u32 = 19;
    pub const WRITE_STREAM: u32 = 20;
    pub const READ_STREAM: u32 = 21;
    pub const WIIU_INSTALL_FROM_FOLDER: u32 = 0x300;

    /// What vcinject Link up to version 13 implements (protocol 0, no `commands` key)
    pub const LEGACY: &[u32] = &[
        INFO,
        TITLES,
        METADATA,
        READ_FILE,
        WRITE_FILE,
        LIST_DIR,
        INSTALL_LEGACY,
        DELETE,
        REBOOT,
        REPORT,
        NOTE,
        METADATA_BATCH,
        PROGRESS,
        RESTART,
        LAUNCH,
    ];
}

pub mod error {
    pub const BAD_REQUEST: u32 = 0xE000_0001;
    pub const NO_MEMORY: u32 = 0xE000_0002;
    pub const TOO_LARGE: u32 = 0xE000_0003;
    pub const UNKNOWN_COMMAND: u32 = 0xE000_0004;
    pub const BUSY: u32 = 0xE000_0005;
    pub const NOT_SUPPORTED: u32 = 0xE000_0006;
    pub const CANCELLED: u32 = 0xE000_0007;
}

/// A non-zero result from the companion app
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemoteError(pub u32);

impl fmt::Display for RemoteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let reason = match self.0 {
            error::BAD_REQUEST => "the console did not understand the request",
            error::NO_MEMORY => "the console ran out of memory",
            error::TOO_LARGE => "too large",
            error::UNKNOWN_COMMAND => "the companion app is too old for this, update it",
            error::BUSY => "the console is busy",
            error::NOT_SUPPORTED => "not supported on this console",
            error::CANCELLED => "cancelled on the console",
            code => return write!(f, "console error 0x{code:08X}"),
        };

        f.write_str(reason)
    }
}

impl std::error::Error for RemoteError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Console {
    N3ds,
    Wii,
    WiiU,
}

impl Console {
    fn from_key(key: &str) -> Option<Self> {
        match key {
            "3ds" => Some(Self::N3ds),
            "wii" => Some(Self::Wii),
            "wiiu" => Some(Self::WiiU),
            _ => None,
        }
    }
}

/// What a companion app says about itself, in discovery and in the info command
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct Info {
    pub keys: HashMap<String, String>,
}

impl Info {
    pub fn parse(text: &str) -> Self {
        let keys = text
            .lines()
            .filter_map(|line| line.split_once('='))
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect();

        Self { keys }
    }

    /// Parses a discovery reply; None if it isn't from a companion app
    pub fn parse_discovery_reply(reply: &[u8]) -> Option<Self> {
        let text = String::from_utf8_lossy(reply);
        let first_line = text.lines().next()?;

        matches!(first_line, "ugbm-link" | "vcinject-link").then(|| Self::parse(&text))
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.keys.get(key).map(String::as_str)
    }

    fn number(&self, key: &str) -> Option<u64> {
        self.get(key)?.parse().ok()
    }

    pub fn app(&self) -> &str {
        self.get("app").unwrap_or_default()
    }

    pub fn version(&self) -> u32 {
        self.number("version").unwrap_or(0) as u32
    }

    pub fn proto(&self) -> u32 {
        self.number("proto").unwrap_or(0) as u32
    }

    /// None for a console this app doesn't know yet
    pub fn console(&self) -> Option<Console> {
        self.get("console")
            .map_or(Some(Console::N3ds), Console::from_key)
    }

    pub fn name(&self) -> Option<&str> {
        self.get("name").filter(|name| !name.is_empty())
    }

    /// A stable ID of the device, to recognise it under another address
    pub fn device_id(&self) -> Option<&str> {
        self.get("id").filter(|id| !id.is_empty())
    }

    pub fn port(&self) -> u16 {
        self.number("port").map_or(PORT, |port| port as u16)
    }

    pub fn supports(&self, command: u32) -> bool {
        match self.get("commands") {
            Some(list) => list
                .split(',')
                .filter_map(|n| n.trim().parse::<u32>().ok())
                .any(|n| n == command),
            None => self.proto() == 0 && command::LEGACY.contains(&command),
        }
    }

    pub fn battery(&self) -> Option<u8> {
        self.number("battery").map(|level| level as u8)
    }

    pub fn charging(&self) -> bool {
        self.get("charging") == Some("1")
    }

    /// Free and total bytes of a storage location; the 3DS's SD card has legacy names
    pub fn storage(&self, location: u8) -> Option<(u64, u64)> {
        let free = self.number(&format!("free.{location}"));
        let total = self.number(&format!("total.{location}"));

        match (free, total) {
            (Some(free), Some(total)) => Some((free, total)),
            _ if self.console() == Some(Console::N3ds) && location == n3ds::LOCATION_SD => {
                Some((self.number("sd_free")?, self.number("sd_total")?))
            }
            _ => None,
        }
    }
}

pub mod n3ds {
    pub const LOCATION_NAND: u8 = 0;
    pub const LOCATION_SD: u8 = 1;
    pub const LOCATION_GAME_CARD: u8 = 2;

    pub const ARCHIVE_SDMC: u32 = 0x9;
    pub const ARCHIVE_EXTDATA: u32 = 0x6;
    pub const ARCHIVE_SYSTEM_SAVEDATA: u32 = 0x8;
    pub const ARCHIVE_NAND_CTR_FS: u32 = 0x5678_90AB;
    pub const ARCHIVE_SAVEDATA_AND_CONTENT: u32 = 0x2345_678A;
    pub const ARCHIVE_USER_SAVEDATA: u32 = 0x5678_90B2;

    /// CFGU_GetSystemModel
    pub fn model_name(model: &str) -> &'static str {
        match model {
            "1" => "3DS XL",
            "2" => "New 3DS",
            "3" => "2DS",
            "4" => "New 3DS XL",
            "5" => "New 2DS XL",
            _ => "3DS",
        }
    }
}

/// A path in the wire format: u32 type, u32 length, bytes
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Path {
    Empty,
    Binary(Vec<u8>),
    Ascii(String),
    Utf16(String),
    Utf8(String),
}

impl Path {
    pub fn binary(words: &[u32]) -> Self {
        Self::Binary(words.iter().flat_map(|w| w.to_le_bytes()).collect())
    }

    fn encode(&self, out: &mut Vec<u8>) {
        let (kind, bytes): (u32, Vec<u8>) = match self {
            Self::Empty => (1, Vec::new()),
            Self::Binary(bytes) => (2, bytes.clone()),
            Self::Ascii(s) => (3, [s.as_bytes(), &[0]].concat()),
            Self::Utf16(s) => (
                4,
                s.encode_utf16()
                    .chain([0])
                    .flat_map(u16::to_le_bytes)
                    .collect(),
            ),
            Self::Utf8(s) => (5, s.as_bytes().to_vec()),
        };

        out.extend_from_slice(&kind.to_le_bytes());
        out.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
        out.extend_from_slice(&bytes);
    }
}

/// Where a path lives: a console-specific archive ID and its archive path
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Archive {
    pub id: u32,
    pub path: Path,
}

impl Archive {
    pub fn new(id: u32, path: Path) -> Self {
        Self { id, path }
    }

    pub fn encode(&self, file: &Path) -> Vec<u8> {
        let mut out = self.id.to_le_bytes().to_vec();
        self.path.encode(&mut out);
        file.encode(&mut out);
        out
    }
}

pub const TITLE_RECORD_SIZE: usize = 40;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Title {
    pub id: u64,
    pub size: u64,
    pub version: u16,
    pub location: u8,
    pub product_code: String,
}

impl Title {
    pub fn parse_list(data: &[u8]) -> Vec<Self> {
        data.as_chunks::<TITLE_RECORD_SIZE>()
            .0
            .iter()
            .map(|record| {
                let code = &record[24..40];
                let code_len = code.iter().position(|&b| b == 0).unwrap_or(code.len());

                Self {
                    id: u64::from_le_bytes(record[0..8].try_into().unwrap()),
                    size: u64::from_le_bytes(record[8..16].try_into().unwrap()),
                    version: u16::from_le_bytes(record[16..18].try_into().unwrap()),
                    location: record[18],
                    product_code: String::from_utf8_lossy(&code[..code_len]).into_owned(),
                }
            })
            .collect()
    }

    pub fn id_payload(&self) -> Vec<u8> {
        let mut out = vec![self.location];
        out.extend_from_slice(&self.id.to_le_bytes());
        out
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirEntry {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
}

impl DirEntry {
    pub fn parse_list(mut data: &[u8]) -> Vec<Self> {
        let mut entries = Vec::new();

        while data.len() >= 16 {
            let is_dir = data[0] != 0;
            let size = u64::from_le_bytes(data[4..12].try_into().unwrap());
            let name_len = u32::from_le_bytes(data[12..16].try_into().unwrap()) as usize;
            let Some(name) = data.get(16..16 + name_len) else {
                break;
            };

            entries.push(Self {
                name: String::from_utf8_lossy(name).into_owned(),
                is_dir,
                size,
            });
            data = &data[16 + name_len..];
        }

        entries
    }
}

/// Splits the answer of the metadata batch command into one entry per title
pub fn parse_metadata_batch(mut data: &[u8]) -> Vec<Option<Vec<u8>>> {
    let mut out = Vec::new();

    while data.len() >= 4 {
        let size = u32::from_le_bytes(data[..4].try_into().unwrap()) as usize;
        let Some(metadata) = data.get(4..4 + size) else {
            break;
        };

        out.push((size > 0).then(|| metadata.to_vec()));
        data = &data[4 + size..];
    }

    out
}

pub fn write_request(w: &mut impl Write, command: u32, payload: &[u8]) -> io::Result<()> {
    let mut frame = Vec::with_capacity(8 + payload.len());
    frame.extend_from_slice(&command.to_le_bytes());
    frame.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    frame.extend_from_slice(payload);
    w.write_all(&frame)
}

#[derive(Debug)]
pub enum AnswerError {
    Io(io::Error),
    Remote(RemoteError),
}

impl From<io::Error> for AnswerError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

pub fn read_answer(r: &mut impl Read) -> Result<Vec<u8>, AnswerError> {
    let mut header = [0; 8];
    r.read_exact(&mut header)?;
    let result = u32::from_le_bytes(header[..4].try_into().unwrap());
    let size = u32::from_le_bytes(header[4..].try_into().unwrap());

    let mut payload = vec![0; size as usize];
    r.read_exact(&mut payload)?;

    match result {
        0 => Ok(payload),
        code => Err(AnswerError::Remote(RemoteError(code))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_discovery_reply() {
        let reply = b"vcinject-link\nport=34013\napp=vcinject-link\nversion=13\nname=Petter\nmodel=1\nsd_free=10\nsd_total=20\n";
        let info = Info::parse_discovery_reply(reply).unwrap();

        assert_eq!(info.version(), 13);
        assert_eq!(info.proto(), 0);
        assert_eq!(info.console(), Some(Console::N3ds));
        assert_eq!(info.name(), Some("Petter"));
        assert!(info.supports(command::METADATA_BATCH));
        assert!(!info.supports(command::INSTALL_STREAM));
        assert_eq!(info.storage(n3ds::LOCATION_SD), Some((10, 20)));
    }

    #[test]
    fn discovery_reply_with_commands() {
        let reply = b"ugbm-link\nproto=1\nconsole=wiiu\ncommands=1, 2,20\nfree.1=5\ntotal.1=9\n";
        let info = Info::parse_discovery_reply(reply).unwrap();

        assert_eq!(info.console(), Some(Console::WiiU));
        assert!(info.supports(command::WRITE_STREAM));
        assert!(!info.supports(command::METADATA));
        assert_eq!(info.storage(1), Some((5, 9)));
    }

    #[test]
    fn not_a_companion() {
        assert_eq!(Info::parse_discovery_reply(b"hello\nport=1"), None);
        assert_eq!(Info::parse("console=switch").console(), None);
    }

    #[test]
    fn title_records() {
        let mut data = Vec::new();
        data.extend_from_slice(&0x0004_0000_0012_3400_u64.to_le_bytes());
        data.extend_from_slice(&1234_u64.to_le_bytes());
        data.extend_from_slice(&5_u16.to_le_bytes());
        data.push(1);
        data.extend_from_slice(&[0; 5]);
        data.extend_from_slice(b"CTR-P-AXXE\0\0\0\0\0\0");

        let titles = Title::parse_list(&data);
        assert_eq!(
            titles,
            vec![Title {
                id: 0x0004_0000_0012_3400,
                size: 1234,
                version: 5,
                location: 1,
                product_code: "CTR-P-AXXE".into(),
            }]
        );
        assert_eq!(titles[0].id_payload()[0], 1);
    }

    #[test]
    fn archive_encoding() {
        let archive = Archive::new(n3ds::ARCHIVE_SDMC, Path::Empty);
        let encoded = archive.encode(&Path::Utf16("/a".into()));

        assert_eq!(
            encoded,
            [
                &9_u32.to_le_bytes()[..],
                &1_u32.to_le_bytes(),
                &0_u32.to_le_bytes(),
                &4_u32.to_le_bytes(),
                &6_u32.to_le_bytes(),
                &[b'/', 0, b'a', 0, 0, 0],
            ]
            .concat()
        );
    }

    #[test]
    fn dir_entries_and_metadata_batch() {
        let mut data = vec![1, 0, 0, 0];
        data.extend_from_slice(&0_u64.to_le_bytes());
        data.extend_from_slice(&3_u32.to_le_bytes());
        data.extend_from_slice(b"3ds");
        assert_eq!(
            DirEntry::parse_list(&data),
            vec![DirEntry {
                name: "3ds".into(),
                is_dir: true,
                size: 0
            }]
        );

        let batch = [&2_u32.to_le_bytes()[..], b"ab", &0_u32.to_le_bytes()].concat();
        assert_eq!(
            parse_metadata_batch(&batch),
            vec![Some(b"ab".to_vec()), None]
        );
    }

    #[test]
    fn frames() {
        let mut wire = Vec::new();
        write_request(&mut wire, command::NOTE, b"hi").unwrap();
        assert_eq!(wire, [11, 0, 0, 0, 2, 0, 0, 0, b'h', b'i']);

        let answer = [0, 0, 0, 0, 1, 0, 0, 0, 42];
        assert_eq!(read_answer(&mut &answer[..]).unwrap(), vec![42]);

        let failed = [4, 0, 0, 0xE0, 0, 0, 0, 0];
        assert!(matches!(
            read_answer(&mut &failed[..]),
            Err(AnswerError::Remote(RemoteError(error::UNKNOWN_COMMAND)))
        ));
    }
}
