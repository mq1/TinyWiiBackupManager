// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: MIT OR Apache-2.0

include!(concat!(env!("OUT_DIR"), "/id_map_meta.rs"));

#[cfg(not(feature = "compress"))]
static DATA: Data =
    unsafe { std::mem::transmute(*include_bytes!(concat!(env!("OUT_DIR"), "/id_map.bin"))) };

#[cfg(feature = "compress")]
static DATA: std::sync::LazyLock<Box<Data>> = std::sync::LazyLock::new(|| {
    const COMPRESSED: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/id_map.bin"));

    let mut buf = Box::<Data>::new_uninit();

    miniz_oxide::inflate::decompress_slice_iter_to_slice(
        unsafe {
            let data = buf.as_mut_ptr().cast();
            let len = std::mem::size_of::<Data>();
            std::slice::from_raw_parts_mut(data, len)
        },
        std::iter::once(COMPRESSED),
        false,
        true,
    )
    .expect("Corrupted compressed data from build.rs");

    unsafe { buf.assume_init() }
});

pub fn get_title(game_id: u32) -> Option<&'static str> {
    let idx = DATA.title_map_ids.binary_search(&game_id).ok()?;

    let title_str = unsafe {
        let start = *DATA.title_map_offsets.get_unchecked(idx) as usize;
        let end = *DATA.title_map_offsets.get_unchecked(idx.unchecked_add(1)) as usize;

        let title_slice = DATA.all_titles.get_unchecked(start..end);
        std::str::from_utf8_unchecked(title_slice)
    };

    Some(title_str)
}

#[cfg(feature = "gamehacking")]
pub fn get_ghid(game_id: u32) -> Option<u32> {
    let idx = DATA.gamehacking_map_ids.binary_search(&game_id).ok()?;
    let ghid = unsafe { *DATA.gamehacking_map_ghids.get_unchecked(idx) };

    Some(ghid)
}

#[cfg(feature = "ascii-titles")]
pub fn get_ascii_title(game_id: u32) -> Option<&'static str> {
    let idx = DATA.ascii_title_map_ids.binary_search(&game_id).ok()?;

    let title_str = unsafe {
        let start = *DATA.ascii_title_map_offsets.get_unchecked(idx) as usize;
        let end = *DATA
            .ascii_title_map_offsets
            .get_unchecked(idx.unchecked_add(1)) as usize;

        let title_slice = DATA.all_titles.get_unchecked(start..end);
        std::str::from_utf8_unchecked(title_slice)
    };

    Some(title_str)
}
