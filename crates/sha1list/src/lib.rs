// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

const BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/sha1_list.bin"));

const CHUNKED: &[[u8; 20]] = {
    let (chunks, remainder) = BYTES.as_chunks();
    assert!(remainder.is_empty());
    chunks
};

pub fn is_known(sha1: &[u8; 20]) -> bool {
    CHUNKED.binary_search(sha1).is_ok()
}
