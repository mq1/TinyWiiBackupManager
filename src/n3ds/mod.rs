// SPDX-FileCopyrightText: 2026 Yanik Fuchs <yanik.fuchs@me.com>
// SPDX-License-Identifier: GPL-3.0-only

//! Nintendo 3DS formats

pub mod cia;
pub mod smdh;

/// Title ID high words of what people think of as games and apps (not system titles, updates, DLC)
const APP_ID_HIGHS: [u32; 2] = [0x0004_0000, 0x0004_0002];

pub fn is_app(title_id: u64) -> bool {
    APP_ID_HIGHS.contains(&((title_id >> 32) as u32))
}
