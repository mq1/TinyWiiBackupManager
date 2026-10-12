use std::io::{self, Read, Seek, Write};

use tracing::warn;

use crate::{
    common::MagicBytes,
    disc::DL_DVD_SIZE,
    read::{DiscMeta, DiscStream},
    util::read::{read_at, read_from, read_u16_be, read_u32_be, read_u64_be, read_vec},
};

#[allow(unused)]
#[repr(u16)]
enum NKitHeaderFlags {
    Size = 0x1,
    Crc32 = 0x2,
    Md5 = 0x4,
    Sha1 = 0x8,
    Xxhash64 = 0x10,
    Key = 0x20,
    Encrypted = 0x40,
    ExtraData = 0x80,
    IndexFile = 0x100,
}

const NKIT_HEADER_V1_FLAGS: u16 = NKitHeaderFlags::Crc32 as u16
    | NKitHeaderFlags::Md5 as u16
    | NKitHeaderFlags::Sha1 as u16
    | NKitHeaderFlags::Xxhash64 as u16;

const fn calc_header_size(version: u8, flags: u16, key_len: u32) -> usize {
    let mut size = 8;
    if version >= 2 {
        // header size + flags
        size += 4;
    }
    if flags & NKitHeaderFlags::Size as u16 != 0 {
        size += 8;
    }
    if flags & NKitHeaderFlags::Crc32 as u16 != 0 {
        size += 4;
    }
    if flags & NKitHeaderFlags::Md5 as u16 != 0 {
        size += 16;
    }
    if flags & NKitHeaderFlags::Sha1 as u16 != 0 {
        size += 20;
    }
    if flags & NKitHeaderFlags::Xxhash64 as u16 != 0 {
        size += 8;
    }
    if flags & NKitHeaderFlags::Key as u16 != 0 {
        size += key_len as usize + 2;
    }
    size
}

#[derive(Debug, Clone)]
pub struct NKitHeader {
    pub version: u8,
    pub size: Option<u64>,
    pub crc32: Option<u32>,
    pub md5: Option<[u8; 16]>,
    pub sha1: Option<[u8; 20]>,
    pub xxh64: Option<u64>,
    /// Bitstream of blocks that are junk data
    pub junk_bits: Option<JunkBits>,
    pub encrypted: bool,
}

impl Default for NKitHeader {
    fn default() -> Self {
        Self {
            version: 2,
            size: None,
            crc32: None,
            md5: None,
            sha1: None,
            xxh64: None,
            junk_bits: None,
            encrypted: false,
        }
    }
}

const VERSION_PREFIX: [u8; 7] = *b"NKIT  v";

impl NKitHeader {
    pub fn try_read_from(
        reader: &mut dyn DiscStream,
        pos: u64,
        block_size: u32,
        has_junk_bits: bool,
    ) -> Option<Self> {
        let magic: MagicBytes = read_at(reader, pos).ok()?;
        if magic == *b"NKIT" {
            let mut reader = ReadAdapter::new(reader, pos);
            match NKitHeader::read_from(&mut reader, block_size, has_junk_bits) {
                Ok(header) => Some(header),
                Err(e) => {
                    warn!("Failed to read NKit header: {}", e);
                    None
                }
            }
        } else {
            None
        }
    }

    pub fn read_from<R>(reader: &mut R, block_size: u32, has_junk_bits: bool) -> io::Result<Self>
    where R: Read + ?Sized {
        let version_string: [u8; 8] = read_from(reader)?;
        if version_string[0..7] != VERSION_PREFIX
            || version_string[7] < b'1'
            || version_string[7] > b'9'
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid NKit header version string",
            ));
        }
        let version = version_string[7] - b'0';
        let header_size = match version {
            1 => calc_header_size(version, NKIT_HEADER_V1_FLAGS, 0) as u16,
            2 => read_u16_be(reader)?,
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Unsupported NKit header version: {}", version),
                ));
            }
        };

        let mut remaining_header_size = header_size as usize - 8;
        if version >= 2 {
            // We read the header size already
            remaining_header_size -= 2;
        }
        let header_bytes = read_vec(reader, remaining_header_size)?;
        let mut inner = &header_bytes[..];

        let flags = if version == 1 { NKIT_HEADER_V1_FLAGS } else { read_u16_be(&mut inner)? };
        let size = (flags & NKitHeaderFlags::Size as u16 != 0)
            .then(|| read_u64_be(&mut inner))
            .transpose()?;
        let crc32 = (flags & NKitHeaderFlags::Crc32 as u16 != 0)
            .then(|| read_u32_be(&mut inner))
            .transpose()?;
        let md5 = (flags & NKitHeaderFlags::Md5 as u16 != 0)
            .then(|| read_from::<[u8; 16], _>(&mut inner))
            .transpose()?;
        let sha1 = (flags & NKitHeaderFlags::Sha1 as u16 != 0)
            .then(|| read_from::<[u8; 20], _>(&mut inner))
            .transpose()?;
        let xxh64 = (flags & NKitHeaderFlags::Xxhash64 as u16 != 0)
            .then(|| read_u64_be(&mut inner))
            .transpose()?;

        let junk_bits =
            if has_junk_bits { Some(JunkBits::read_from(reader, block_size)?) } else { None };

        let encrypted = flags & NKitHeaderFlags::Encrypted as u16 != 0;

        Ok(Self { version, size, crc32, md5, sha1, xxh64, junk_bits, encrypted })
    }

    pub fn is_junk_block(&self, block: u32) -> Option<bool> {
        self.junk_bits.as_ref().map(|v| v.get(block))
    }

    pub fn apply(&self, meta: &mut DiscMeta) {
        meta.needs_hash_recovery |= self.junk_bits.is_some();
        meta.lossless |= self.size.is_some() && self.junk_bits.is_some();
        meta.disc_size = meta.disc_size.or(self.size);
        meta.crc32 = self.crc32;
        meta.md5 = self.md5;
        meta.sha1 = self.sha1;
        meta.xxh64 = self.xxh64;
    }

    fn calc_flags(&self) -> u16 {
        let mut flags = 0;
        if self.size.is_some() {
            flags |= NKitHeaderFlags::Size as u16;
        }
        if self.crc32.is_some() {
            flags |= NKitHeaderFlags::Crc32 as u16;
        }
        if self.md5.is_some() {
            flags |= NKitHeaderFlags::Md5 as u16;
        }
        if self.sha1.is_some() {
            flags |= NKitHeaderFlags::Sha1 as u16;
        }
        if self.xxh64.is_some() {
            flags |= NKitHeaderFlags::Xxhash64 as u16;
        }
        if self.encrypted {
            flags |= NKitHeaderFlags::Encrypted as u16;
        }
        flags
    }

    pub fn write_to<W>(&self, w: &mut W) -> io::Result<()>
    where W: Write + ?Sized {
        w.write_all(&VERSION_PREFIX)?;
        w.write_all(&[b'0' + self.version])?;
        let flags = self.calc_flags();
        match self.version {
            1 => {}
            2 => {
                let header_size = calc_header_size(self.version, flags, 0) as u16;
                w.write_all(&header_size.to_be_bytes())?;
                w.write_all(&flags.to_be_bytes())?;
            }
            version => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Unsupported NKit header version: {}", version),
                ));
            }
        };
        if let Some(size) = self.size {
            w.write_all(&size.to_be_bytes())?;
        }
        if let Some(crc32) = self.crc32 {
            w.write_all(&crc32.to_be_bytes())?;
        } else if self.version == 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Missing CRC32 in NKit v1 header",
            ));
        }
        if let Some(md5) = self.md5 {
            w.write_all(&md5)?;
        } else if self.version == 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Missing MD5 in NKit v1 header",
            ));
        }
        if let Some(sha1) = self.sha1 {
            w.write_all(&sha1)?;
        } else if self.version == 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Missing SHA1 in NKit v1 header",
            ));
        }
        if let Some(xxh64) = self.xxh64 {
            w.write_all(&xxh64.to_be_bytes())?;
        } else if self.version == 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Missing XXHash64 in NKit header",
            ));
        }
        if let Some(junk_bits) = &self.junk_bits {
            junk_bits.write_to(w)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct JunkBits(Vec<u8>);

impl JunkBits {
    pub fn new(block_size: u32) -> Self { Self(vec![0; Self::len(block_size)]) }

    pub fn read_from<R>(reader: &mut R, block_size: u32) -> io::Result<Self>
    where R: Read + ?Sized {
        Ok(Self(read_vec(reader, Self::len(block_size))?))
    }

    pub fn write_to<W>(&self, w: &mut W) -> io::Result<()>
    where W: Write + ?Sized {
        w.write_all(&self.0)
    }

    pub fn set(&mut self, block: u32, is_junk: bool) {
        let Some(byte) = self.0.get_mut((block / 8) as usize) else {
            return;
        };
        if is_junk {
            *byte |= 1 << (7 - (block & 7));
        } else {
            *byte &= !(1 << (7 - (block & 7)));
        }
    }

    pub fn get(&self, block: u32) -> bool {
        let Some(&byte) = self.0.get((block / 8) as usize) else {
            return false;
        };
        byte & (1 << (7 - (block & 7))) != 0
    }

    fn len(block_size: u32) -> usize {
        DL_DVD_SIZE.div_ceil(block_size as u64).div_ceil(8) as usize
    }
}

pub struct ReadAdapter<'a> {
    reader: &'a mut dyn DiscStream,
    pos: u64,
}

impl<'a> ReadAdapter<'a> {
    pub fn new(reader: &'a mut dyn DiscStream, offset: u64) -> Self { Self { reader, pos: offset } }
}

impl Read for ReadAdapter<'_> {
    fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
        Err(io::Error::from(io::ErrorKind::Unsupported))
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        self.reader.read_exact_at(buf, self.pos)?;
        self.pos += buf.len() as u64;
        Ok(())
    }
}

impl Seek for ReadAdapter<'_> {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        self.pos = match pos {
            io::SeekFrom::Start(pos) => pos,
            io::SeekFrom::End(v) => self.reader.stream_len()?.saturating_add_signed(v),
            io::SeekFrom::Current(v) => self.pos.saturating_add_signed(v),
        };
        Ok(self.pos)
    }
}
