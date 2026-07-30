// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

const BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/sha1_list.bin"));

pub fn is_known(sha1: &[u8; 20]) -> bool {
    unsafe { BYTES.as_chunks_unchecked().binary_search(sha1).is_ok() }
}
