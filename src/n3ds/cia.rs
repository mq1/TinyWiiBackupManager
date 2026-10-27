// SPDX-FileCopyrightText: 2026 Yanik Fuchs <yanik.fuchs@me.com>
// SPDX-License-Identifier: GPL-3.0-only

//! What a CIA says about the title inside: its ID, version and SMDH (from the meta section)

use crate::n3ds::smdh::{SMDH_SIZE, is_smdh};
use std::io::{self, Read, Seek, SeekFrom};

const META_SMDH_OFFSET: u64 = 0x400;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CiaInfo {
    pub title_id: u64,
    pub version: u16,
    pub smdh: Option<Vec<u8>>,
}

fn align64(n: u64) -> u64 {
    n.div_ceil(64) * 64
}

fn u32_le(buf: &[u8], at: usize) -> u64 {
    u64::from(u32::from_le_bytes(buf[at..at + 4].try_into().unwrap()))
}

/// Signature and padding before a TMD's header, by signature type
fn signature_size(kind: u32) -> Option<u64> {
    match kind {
        0x10000 | 0x10003 => Some(4 + 0x200 + 0x3C),
        0x10001 | 0x10004 => Some(4 + 0x100 + 0x3C),
        0x10002 | 0x10005 => Some(4 + 0x3C + 0x40),
        _ => None,
    }
}

pub fn read(mut f: impl Read + Seek) -> io::Result<CiaInfo> {
    let invalid = || io::Error::new(io::ErrorKind::InvalidData, "not a CIA");

    let mut header = [0; 0x20];
    f.read_exact(&mut header)?;
    let header_size = u32_le(&header, 0);
    let cert_size = u32_le(&header, 0x08);
    let ticket_size = u32_le(&header, 0x0C);
    let tmd_size = u32_le(&header, 0x10);
    let meta_size = u32_le(&header, 0x14);
    let content_size = u64::from_le_bytes(header[0x18..0x20].try_into().unwrap());
    if header_size != 0x2020 {
        return Err(invalid());
    }

    let tmd_offset = align64(header_size) + align64(cert_size) + align64(ticket_size);
    f.seek(SeekFrom::Start(tmd_offset))?;
    let mut kind = [0; 4];
    f.read_exact(&mut kind)?;
    let sig = signature_size(u32::from_be_bytes(kind)).ok_or_else(invalid)?;

    let mut tmd = [0; 0xA0];
    f.seek(SeekFrom::Start(tmd_offset + sig))?;
    f.read_exact(&mut tmd)?;
    let title_id = u64::from_be_bytes(tmd[0x4C..0x54].try_into().unwrap());
    let version = u16::from_be_bytes(tmd[0x9C..0x9E].try_into().unwrap());

    let smdh = if meta_size >= META_SMDH_OFFSET + SMDH_SIZE as u64 {
        let meta_offset = tmd_offset + align64(tmd_size) + align64(content_size);
        let mut smdh = vec![0; SMDH_SIZE];
        f.seek(SeekFrom::Start(meta_offset + META_SMDH_OFFSET))?;
        f.read_exact(&mut smdh)?;
        is_smdh(&smdh).then_some(smdh)
    } else {
        None
    };

    Ok(CiaInfo {
        title_id,
        version,
        smdh,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn reads_title_id_version_and_smdh() {
        let (cert, ticket, tmd, content) = (0xA00_u64, 0x350_u64, 0xB34_u64, 0x1000_u64);
        let meta = META_SMDH_OFFSET + SMDH_SIZE as u64;

        let mut cia = vec![0; 0x2020];
        cia[0..4].copy_from_slice(&0x2020_u32.to_le_bytes());
        cia[0x08..0x0C].copy_from_slice(&(cert as u32).to_le_bytes());
        cia[0x0C..0x10].copy_from_slice(&(ticket as u32).to_le_bytes());
        cia[0x10..0x14].copy_from_slice(&(tmd as u32).to_le_bytes());
        cia[0x14..0x18].copy_from_slice(&(meta as u32).to_le_bytes());
        cia[0x18..0x20].copy_from_slice(&content.to_le_bytes());

        let tmd_offset = (align64(0x2020) + align64(cert) + align64(ticket)) as usize;
        let meta_offset = tmd_offset + (align64(tmd) + align64(content)) as usize;
        cia.resize(meta_offset + meta as usize, 0);

        cia[tmd_offset..tmd_offset + 4].copy_from_slice(&0x10004_u32.to_be_bytes());
        let header = tmd_offset + 0x140;
        cia[header + 0x4C..header + 0x54].copy_from_slice(&0x0004_0000_0F7C_1A00_u64.to_be_bytes());
        cia[header + 0x9C..header + 0x9E].copy_from_slice(&3_u16.to_be_bytes());
        let smdh = meta_offset + META_SMDH_OFFSET as usize;
        cia[smdh..smdh + 4].copy_from_slice(b"SMDH");

        let info = read(Cursor::new(cia)).unwrap();
        assert_eq!(info.title_id, 0x0004_0000_0F7C_1A00);
        assert_eq!(info.version, 3);
        assert!(info.smdh.is_some());

        assert!(read(Cursor::new(vec![0; 0x40])).is_err());
    }
}
