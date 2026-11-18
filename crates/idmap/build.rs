// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::{
    fs::{self, File},
    io::{BufWriter, Write},
    path::PathBuf,
};

struct Data<'a> {
    titles: Vec<(u32, &'a str)>,

    #[cfg(feature = "ascii-titles")]
    ascii_titles: Vec<(u32, std::borrow::Cow<'a, str>)>,

    #[cfg(feature = "gamehacking")]
    gamehacking_map: Vec<(u32, u32)>,
}

impl<'a> Data<'a> {
    fn write(&self, writer: &mut dyn Write, meta_writer: &mut dyn Write) {
        let endianness = std::env::var("CARGO_CFG_TARGET_ENDIAN").unwrap();
        let encode_u32 = move |value: u32| match endianness.as_str() {
            "little" => value.to_le_bytes(),
            "big" => value.to_be_bytes(),
            _ => unreachable!(),
        };

        meta_writer.write_all(b"#[repr(C)]\nstruct Data {").unwrap();

        let mut all_titles = String::new();

        for (id, _) in &self.titles {
            writer.write_all(&encode_u32(*id)).unwrap();
        }
        write!(meta_writer, "title_map_ids: [u32; {}],", self.titles.len()).unwrap();

        for (_, title) in &self.titles {
            let pos = u32::try_from(all_titles.len()).unwrap();
            writer.write_all(&encode_u32(pos)).unwrap();
            all_titles.push_str(title);
        }
        let pos = u32::try_from(all_titles.len()).unwrap();
        writer.write_all(&encode_u32(pos)).unwrap();
        write!(
            meta_writer,
            "title_map_offsets: [u32; {}],",
            self.titles.len() + 1
        )
        .unwrap();

        #[cfg(feature = "ascii-titles")]
        {
            for (id, _) in &self.ascii_titles {
                writer.write_all(&encode_u32(*id)).unwrap();
            }
            write!(
                meta_writer,
                "ascii_title_map_ids: [u32; {}],",
                self.ascii_titles.len()
            )
            .unwrap();

            for (_, title) in &self.ascii_titles {
                let pos = u32::try_from(all_titles.len()).unwrap();
                writer.write_all(&encode_u32(pos)).unwrap();
                all_titles.push_str(title);
            }
            let pos = u32::try_from(all_titles.len()).unwrap();
            writer.write_all(&encode_u32(pos)).unwrap();
            write!(
                meta_writer,
                "ascii_title_map_offsets: [u32; {}],",
                self.ascii_titles.len() + 1
            )
            .unwrap();
        }

        #[cfg(feature = "gamehacking")]
        {
            for (id, _) in &self.gamehacking_map {
                writer.write_all(&encode_u32(*id)).unwrap();
            }
            write!(
                meta_writer,
                "gamehacking_map_ids: [u32; {}],",
                self.gamehacking_map.len()
            )
            .unwrap();

            for (_, ghid) in &self.gamehacking_map {
                writer.write_all(&encode_u32(*ghid)).unwrap();
            }
            write!(
                meta_writer,
                "gamehacking_map_ghids: [u32; {}],",
                self.gamehacking_map.len()
            )
            .unwrap();
        }

        // pad the titles to 4 bytes
        while all_titles.len() % 4 != 0 {
            all_titles.push('\0');
        }

        writer.write_all(all_titles.as_bytes()).unwrap();
        write!(meta_writer, "all_titles: [u8; {}]}}", all_titles.len()).unwrap();
    }
}

fn parse_titles_txt(content: &str) -> Vec<(u32, &str)> {
    let mut entries = Vec::with_capacity(16_384);

    for line in content.lines().skip(1) {
        let (id, title) = line.split_once(" = ").unwrap();
        let id = u32::from_str_radix(id, 36).unwrap();

        entries.push((id, title));
    }

    entries
}

#[cfg(feature = "ascii-titles")]
fn make_ascii_map<'a>(
    title_map: &[(u32, &str)],
    en_title_map: &[(u32, &'a str)],
) -> Vec<(u32, std::borrow::Cow<'a, str>)> {
    let mut entries = Vec::with_capacity(4096);

    for ((id, title), (en_id, en_title)) in
        title_map.iter().copied().zip(en_title_map.iter().copied())
    {
        assert_eq!(id, en_id);

        if title.is_ascii() {
            // original title is already ascii, don't add an entry
            // we handle this by falling back to the original title
            continue;
        }

        if en_title.is_ascii() {
            entries.push((en_id, en_title.into()));
        } else {
            let mut ascii_title = en_title
                .chars()
                .filter(char::is_ascii)
                .skip_while(char::is_ascii_whitespace)
                .collect::<String>();

            ascii_title.truncate(ascii_title.trim_end().len());

            assert!(!ascii_title.is_empty());
            entries.push((en_id, ascii_title.into()));
        }
    }

    entries
}

#[cfg(feature = "gamehacking")]
fn parse_gamehacking_ids() -> Vec<(u32, u32)> {
    const GHID_ANCHOR: &str = "href=\"/game/";
    const GAMEID_ANCHOR: &str = "<td class=\"text-center\">";

    let mut entries = Vec::with_capacity(2048);

    for i in 0..=70 {
        let filename = format!("assets/gamehacking/GameHacking.org - WII - Page {i}.html");
        let content = fs::read_to_string(&filename).unwrap();

        let mut current_slice = &content[..];
        while let Some(ghid_pos) = current_slice.find(GHID_ANCHOR) {
            current_slice = &current_slice[ghid_pos + GHID_ANCHOR.len()..];

            let quote_pos = current_slice.find('"').unwrap();
            let ghid_str = &current_slice[..quote_pos];
            let ghid = ghid_str.parse().unwrap();
            if ghid == 0 {
                continue;
            }

            let gameid_pos = current_slice.find(GAMEID_ANCHOR).unwrap();
            current_slice = &current_slice[gameid_pos + GAMEID_ANCHOR.len()..];
            let td_close_pos = current_slice.find('<').unwrap();
            let gameid_str = current_slice[..td_close_pos].trim();
            let gameid_str_len = gameid_str.len();
            if gameid_str_len != 4 && gameid_str_len != 6 {
                continue;
            }

            let gameid = u32::from_str_radix(gameid_str, 36).unwrap();

            entries.push((gameid, ghid));
        }
    }

    entries.sort_unstable_by_key(|(id, _)| *id);
    entries.dedup_by_key(|(id, _)| *id);

    entries
}

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=assets/**");

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    let titles_txt = fs::read_to_string("assets/wiitdb.txt").unwrap();
    let mut titles = parse_titles_txt(&titles_txt);

    #[cfg(feature = "ascii-titles")]
    let en_titles_txt = fs::read_to_string("assets/wiitdb-en.txt").unwrap();
    #[cfg(feature = "ascii-titles")]
    let en_titles = parse_titles_txt(&en_titles_txt);
    #[cfg(feature = "ascii-titles")]
    let mut ascii_titles = make_ascii_map(&titles, &en_titles);

    titles.sort_unstable_by_key(|(id, _)| *id);
    titles.dedup_by_key(|(id, _)| *id);

    #[cfg(feature = "ascii-titles")]
    {
        ascii_titles.sort_unstable_by_key(|(id, _)| *id);
        ascii_titles.dedup_by_key(|(id, _)| *id);
    }

    #[cfg(feature = "gamehacking")]
    let gamehacking_map = parse_gamehacking_ids();

    let data = Data {
        titles,
        #[cfg(feature = "ascii-titles")]
        ascii_titles,
        #[cfg(feature = "gamehacking")]
        gamehacking_map,
    };

    let out_path = out_dir.join("id_map.bin");
    let out_file = File::create(out_path).unwrap();
    let mut out_writer = BufWriter::new(out_file);

    let meta_out_path = out_dir.join("id_map_meta.rs");
    let meta_out_file = File::create(meta_out_path).unwrap();
    let mut meta_out_writer = BufWriter::new(meta_out_file);

    #[cfg(feature = "compress")]
    {
        let mut uncompressed = Vec::new();
        data.write(&mut uncompressed, &mut meta_out_writer);

        let compressed = miniz_oxide::deflate::compress_to_vec(&uncompressed, 9);
        out_writer.write_all(&compressed).unwrap();
    }

    #[cfg(not(feature = "compress"))]
    data.write(&mut out_writer, &mut meta_out_writer);

    out_writer.flush().unwrap();
    meta_out_writer.flush().unwrap();
}
