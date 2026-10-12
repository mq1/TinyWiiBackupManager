use tracing::instrument;
use zerocopy::{FromZeros, IntoBytes};

use crate::{
    common::HashBytes,
    disc::{
        SECTOR_GROUP_SIZE, SECTOR_SIZE,
        wii::{HASHES_SIZE, SECTOR_DATA_SIZE},
    },
    util::{array_ref, array_ref_mut, digest::sha1_hash},
};

/// Hashes for a single sector group (64 sectors).
#[derive(Clone, FromZeros)]
pub struct GroupHashes {
    pub h3_hash: HashBytes,
    pub h2_hashes: [HashBytes; 8],
    pub h1_hashes: [HashBytes; 64],
    pub h0_hashes: [HashBytes; 1984],
}

impl GroupHashes {
    #[inline]
    pub fn hashes_for_sector(
        &self,
        sector: usize,
    ) -> (&[HashBytes; 31], &[HashBytes; 8], &[HashBytes; 8]) {
        let h1_hashes = array_ref![self.h1_hashes, sector & !7, 8];
        let h0_hashes = array_ref![self.h0_hashes, sector * 31, 31];
        (h0_hashes, h1_hashes, &self.h2_hashes)
    }

    #[inline]
    pub fn apply(&self, sector_data: &mut [u8; SECTOR_SIZE], sector: usize) {
        let (h0_hashes, h1_hashes, h2_hashes) = self.hashes_for_sector(sector);
        array_ref_mut![sector_data, 0, 0x26C].copy_from_slice(h0_hashes.as_bytes());
        array_ref_mut![sector_data, 0x280, 0xA0].copy_from_slice(h1_hashes.as_bytes());
        array_ref_mut![sector_data, 0x340, 0xA0].copy_from_slice(h2_hashes.as_bytes());
    }
}

pub const NUM_H0_HASHES: usize = SECTOR_DATA_SIZE / HASHES_SIZE;

#[instrument(skip_all)]
pub fn hash_sector_group(
    sector_group: &[u8; SECTOR_GROUP_SIZE],
    ignore_existing: bool,
) -> Box<GroupHashes> {
    let mut result = GroupHashes::new_box_zeroed().unwrap();
    for (h2_index, h2_hash) in result.h2_hashes.iter_mut().enumerate() {
        let out_h1_hashes = array_ref_mut![result.h1_hashes, h2_index * 8, 8];
        for (h1_index, h1_hash) in out_h1_hashes.iter_mut().enumerate() {
            let sector = h1_index + h2_index * 8;
            let out_h0_hashes =
                array_ref_mut![result.h0_hashes, sector * NUM_H0_HASHES, NUM_H0_HASHES];
            if !ignore_existing
                && array_ref![sector_group, sector * SECTOR_SIZE, 20].iter().any(|&v| v != 0)
            {
                // Hash block already present, use it
                out_h0_hashes.as_mut_bytes().copy_from_slice(array_ref![
                    sector_group,
                    sector * SECTOR_SIZE,
                    0x26C
                ]);
            } else {
                for (h0_index, h0_hash) in out_h0_hashes.iter_mut().enumerate() {
                    *h0_hash = sha1_hash(array_ref![
                        sector_group,
                        sector * SECTOR_SIZE + HASHES_SIZE + h0_index * HASHES_SIZE,
                        HASHES_SIZE
                    ]);
                }
            }
            *h1_hash = sha1_hash(out_h0_hashes.as_bytes());
        }
        *h2_hash = sha1_hash(out_h1_hashes.as_bytes());
    }
    result.h3_hash = sha1_hash(result.h2_hashes.as_bytes());
    result
}
