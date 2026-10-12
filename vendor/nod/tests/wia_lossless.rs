//! Synthetic Wii discs exercise lossless preservation of noncanonical hash areas.

use std::{io::Read, sync::Arc};

use aes::cipher::{BlockModeEncrypt, KeyIvInit, block_padding::NoPadding};
use nod::{
    common::{Compression, Format},
    disc::{SECTOR_SIZE, WII_MAGIC, wii::Ticket},
    read::{DiscOptions, DiscReader},
    write::{DiscWriter, FormatOptions, ProcessOptions},
};
use sha1::{Digest, Sha1};
use zerocopy::FromBytes;

const PARTITION: usize = 0x50000;
const DATA: usize = PARTITION + 0x20000;
// Two complete hash groups and a partial third group. The management/data split
// falls after the first group, so 4 MiB chunks also require empty trailing lists.
const SECTORS: usize = 135;
const HASHES: usize = 0x400;

fn put_u32(data: &mut [u8], offset: usize, value: u32) {
    data[offset..offset + 4].copy_from_slice(&value.to_be_bytes());
}

fn encrypt(key: &[u8; 16], iv: &[u8; 16], data: &mut [u8]) {
    let len = data.len();
    cbc::Encryptor::<aes::Aes128>::new(key.into(), iv.into())
        .encrypt_padded::<NoPadding>(data, len)
        .unwrap();
}

fn synthetic_wii(encrypted: bool, high_entropy: bool) -> Vec<u8> {
    let mut image = vec![0u8; DATA + SECTORS * SECTOR_SIZE];
    image[..6].copy_from_slice(b"SYNX01");
    image[0x18..0x1c].copy_from_slice(&WII_MAGIC);
    image[0x20..0x32].copy_from_slice(b"Synthetic Wii disc");
    image[0x61] = u8::from(!encrypted);
    put_u32(&mut image, 0x40000, 1);
    put_u32(&mut image, 0x40004, 0x40020 / 4);
    put_u32(&mut image, 0x40020, PARTITION as u32 / 4);

    let issuer = b"Root-CA00000001-XS00000003";
    image[PARTITION + 0x140..PARTITION + 0x140 + issuer.len()].copy_from_slice(issuer);
    put_u32(&mut image, PARTITION + 0x2b4, 0x8000 / 4);
    put_u32(&mut image, PARTITION + 0x2b8, 0x20000 / 4);
    put_u32(&mut image, PARTITION + 0x2bc, (SECTORS * SECTOR_SIZE / 4) as u32);
    // H3 is raw partition metadata; it must be preserved rather than repaired.
    image[PARTITION + 0x8000..DATA].fill(0xa7);

    let mut random = 0x0123_4567_89ab_cdefu64;
    for (offset, byte) in image[DATA..].iter_mut().enumerate() {
        random ^= random << 13;
        random ^= random >> 7;
        random ^= random << 17;
        *byte = if high_entropy || offset % SECTOR_SIZE < HASHES {
            random as u8
        } else {
            (offset.wrapping_mul(7) >> 8) as u8
        };
    }

    // Valid partition boot header and a minimal FST containing only its root.
    // All 1024 hash bytes in every sector remain deliberately noncanonical,
    // including H0/H1/H2, all padding regions, sector 63 and the final sector.
    let header = image[..0x400].to_vec();
    let boot = DATA + HASHES;
    image[boot..boot + 0x440].fill(0);
    image[boot..boot + header.len()].copy_from_slice(&header);
    put_u32(&mut image, boot + 0x424, 0x1000 / 4);
    put_u32(&mut image, boot + 0x428, 16 / 4);
    put_u32(&mut image, boot + 0x42c, 16 / 4);
    image[boot + 0x1000..boot + 0x1010].fill(0);
    image[boot + 0x1000] = 1;
    put_u32(&mut image, boot + 0x1008, 1);

    if encrypted {
        let ticket = Ticket::read_from_bytes(&image[PARTITION..PARTITION + 0x2a4]).unwrap();
        let key = ticket.decrypt_title_key().unwrap();
        for sector in image[DATA..].chunks_exact_mut(SECTOR_SIZE) {
            encrypt(&key, &[0; 16], &mut sector[..HASHES]);
            let iv: [u8; 16] = sector[0x3d0..0x3e0].try_into().unwrap();
            encrypt(&key, &iv, &mut sector[HASHES..]);
        }
    }
    image
}

fn assert_lossless(
    format: Format,
    compression: Compression,
    chunk_size: u32,
    encrypted: bool,
    high_entropy: bool,
) {
    let original = synthetic_wii(encrypted, high_entropy);
    assert_image_lossless(original, format, compression, chunk_size, true);
}

fn assert_image_lossless(
    original: Vec<u8>,
    format: Format,
    compression: Compression,
    chunk_size: u32,
    threaded: bool,
) {
    let encrypted = original[0x61] == 0;
    let reader = DiscReader::new_stream(
        Box::new(Arc::<[u8]>::from(original.clone())),
        &DiscOptions {
            #[cfg(feature = "threading")]
            preloader_threads: 2,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(!reader.partitions().is_empty());
    assert!(reader.partitions().iter().all(|partition| partition.fst().is_some()));
    let writer =
        DiscWriter::new(reader, &FormatOptions { format, compression, block_size: chunk_size })
            .unwrap();
    let mut output = Vec::new();
    let finalization = writer
        .process(
            |data, _, _| {
                output.extend_from_slice(&data);
                Ok(())
            },
            &ProcessOptions {
                #[cfg(feature = "threading")]
                processor_threads: if threaded { 2 } else { 0 },
                ..Default::default()
            },
        )
        .unwrap();
    output[..finalization.header.len()].copy_from_slice(&finalization.header);
    let mut roundtrip =
        DiscReader::new_stream(Box::new(Arc::<[u8]>::from(output)), &DiscOptions::default())
            .unwrap();
    let mut restored = Vec::new();
    roundtrip.read_to_end(&mut restored).unwrap();
    assert_eq!(restored.len(), original.len());
    let mismatch = original.iter().zip(&restored).position(|(a, b)| a != b);
    assert!(
        mismatch.is_none(),
        "{format:?}/{compression:?}, chunk {chunk_size:#x}, encrypted={encrypted}: first changed byte {mismatch:#x?}"
    );
}

// Independently derive the 64 canonical hash areas, treating the missing tail
// of a short partition as zeroes. This intentionally does not call nod's helper.
fn canonical_hash_areas(partition_data: &[u8]) -> Vec<[u8; HASHES]> {
    let mut h0 = vec![[0u8; 0x26c]; 64];
    for (sector, hashes) in h0.iter_mut().enumerate() {
        for (block, hash) in hashes.chunks_exact_mut(20).enumerate() {
            let offset = sector * SECTOR_SIZE + HASHES + block * HASHES;
            let data = partition_data.get(offset..offset + HASHES).unwrap_or(&[0; HASHES]);
            hash.copy_from_slice(&Sha1::digest(data));
        }
    }
    let mut h1 = [0u8; 64 * 20];
    for (hash, sector_hashes) in h1.chunks_exact_mut(20).zip(&h0) {
        hash.copy_from_slice(&Sha1::digest(sector_hashes));
    }
    let mut h2 = [0u8; 8 * 20];
    for (hash, subgroup_hashes) in h2.chunks_exact_mut(20).zip(h1.chunks_exact(8 * 20)) {
        hash.copy_from_slice(&Sha1::digest(subgroup_hashes));
    }
    let mut result = vec![[0u8; HASHES]; 64];
    for (sector, hashes) in result.iter_mut().enumerate() {
        hashes[..0x26c].copy_from_slice(&h0[sector]);
        let subgroup = (sector / 8) * 8 * 20;
        hashes[0x280..0x320].copy_from_slice(&h1[subgroup..subgroup + 8 * 20]);
        hashes[0x340..0x3e0].copy_from_slice(&h2);
    }
    result
}

#[test]
fn adjacent_short_partitions_do_not_share_reconstructed_hash_cache() {
    const SHORT_SECTORS: usize = 7;
    let second_partition = DATA + SHORT_SECTORS * SECTOR_SIZE;
    let second_data = second_partition + 0x20000;
    let mut original = synthetic_wii(false, false);
    original.truncate(second_partition);
    put_u32(&mut original, PARTITION + 0x2bc, (SHORT_SECTORS * SECTOR_SIZE / 4) as u32);
    let hashes = canonical_hash_areas(&original[DATA..]);
    for (sector, hash) in hashes.iter().take(SHORT_SECTORS).enumerate() {
        let start = DATA + sector * SECTOR_SIZE;
        original[start..start + HASHES].copy_from_slice(hash);
    }
    let metadata = original[PARTITION..DATA].to_vec();
    let partition_data = original[DATA..].to_vec();
    original.extend_from_slice(&metadata);
    original.extend_from_slice(&partition_data);
    put_u32(&mut original, 0x40000, 2);
    put_u32(&mut original, 0x40028, second_partition as u32 / 4);
    put_u32(&mut original, 0x4002c, 1);

    // Match precisely the stale canonical hashes a range-only cache would use.
    // An all-random hash area would hide this bug by forcing every exception.
    let stale_sector = (second_data - DATA) / SECTOR_SIZE;
    assert!(stale_sector + SHORT_SECTORS < 64);
    for sector in 0..SHORT_SECTORS {
        let start = second_data + sector * SECTOR_SIZE;
        original[start..start + HASHES].copy_from_slice(&hashes[stale_sector + sector]);
    }
    assert_image_lossless(original, Format::Rvz, Compression::None, 0x20000, false);
}

#[test]
fn rvz_32k_uncompressed_preserves_plaintext_hashes_and_padding() {
    assert_lossless(Format::Rvz, Compression::None, 0x8000, false, false);
}

#[cfg(feature = "compress-zstd")]
#[test]
fn rvz_128k_zstd_preserves_encrypted_hashes_and_padding() {
    assert_lossless(Format::Rvz, Compression::Zstandard(1), 0x20000, true, false);
}

#[test]
fn rvz_2m_uncompressed_preserves_sector_63_and_partial_group() {
    assert_lossless(Format::Rvz, Compression::None, 0x200000, true, false);
}

#[cfg(feature = "compress-zstd")]
#[test]
fn rvz_4m_zstd_preserves_multiple_exception_lists() {
    assert_lossless(Format::Rvz, Compression::Zstandard(1), 0x400000, false, false);
}

#[cfg(feature = "compress-zstd")]
#[test]
fn rvz_32k_zstd_uncompressed_fallback_preserves_exceptions() {
    assert_lossless(Format::Rvz, Compression::Zstandard(1), 0x8000, true, true);
}

#[test]
fn wia_4m_uncompressed_preserves_multiple_and_trailing_exception_lists() {
    assert_lossless(Format::Wia, Compression::None, 0x400000, false, false);
}

#[cfg(feature = "compress-lzma")]
#[test]
fn wia_2m_lzma2_allows_maximum_exceptions_with_incompressible_data() {
    assert_lossless(Format::Wia, Compression::Lzma2(1), 0x200000, true, true);
}

#[cfg(feature = "compress-bzip2")]
#[test]
fn wia_2m_bzip2_preserves_plaintext_hashes_and_padding() {
    assert_lossless(Format::Wia, Compression::Bzip2(1), 0x200000, false, false);
}
