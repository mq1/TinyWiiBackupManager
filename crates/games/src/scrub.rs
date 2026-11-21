use anyhow::{Context, Result, anyhow};
use nod::{
    common::Format,
    read::{DiscOptions, DiscReader},
    write::{DiscWriter, FormatOptions, ProcessOptions, ScrubLevel},
};
use split_read::SharedSplitFileReader;
use split_write::SplitWriter;
use std::{
    ffi::OsStr,
    fs,
    io::{BufWriter, Write},
};
use twbm_common::config_contents::ConfigContents;

use crate::{
    OPTIMAL_THREADS,
    disc_stream::SplitFileDiscStream,
    game::Game,
    import::{BUF_SIZE, SPLIT_SIZE, get_filename},
    multi_disc::open_multi_disc_files,
};

pub fn scrub_game(
    game: &Game,
    config: &ConfigContents,
    has_4gib_filesize_limit: bool,
    send_progress: impl Fn(String),
) -> Result<String> {
    let disc_path = game.get_disc_path().context("disc not found")?;

    let game_dir_name = game
        .path()
        .file_name()
        .and_then(OsStr::to_str)
        .context("invalid path")?;

    let tmp_game_dir_name = format!("{game_dir_name} SCRUB");
    let tmp_game_dir = game.path().with_file_name(tmp_game_dir_name);
    fs::create_dir(&tmp_game_dir)?;

    let format_opts = FormatOptions::new(Format::Wbfs);

    let process_opts = ProcessOptions {
        processor_threads: OPTIMAL_THREADS.processor,
        scrub: ScrubLevel::UpdatePartition,
        digest_crc32: true,
        digest_md5: false,
        digest_sha1: true,
        digest_xxh64: true,
    };

    {
        let must_split = config.always_split || has_4gib_filesize_limit;
        let split_size = must_split.then_some(SPLIT_SIZE);

        let disc_reader = {
            let disc_opts = DiscOptions {
                preloader_threads: OPTIMAL_THREADS.preloader,
                ..Default::default()
            };

            let disc_files = open_multi_disc_files(&disc_path)?;
            let disc_files_reader = SharedSplitFileReader::new(disc_files)?;
            DiscReader::new_stream(Box::new(SplitFileDiscStream(disc_files_reader)), &disc_opts)?
        };

        let disc_writer = DiscWriter::new(disc_reader, &format_opts)?;

        let out_writer = SplitWriter::create(
            &tmp_game_dir,
            |part| get_filename(game.id(), true, part, 0, "wbfs", "iso", must_split),
            split_size,
        )?;
        let mut out_writer = BufWriter::with_capacity(BUF_SIZE, out_writer);

        let mut prev_percentage = 100;
        let finalization = disc_writer.process(
            |data, progress, total| {
                out_writer.write_all(&data)?;

                let progress_percentage = progress * 100 / total;
                if progress_percentage != prev_percentage {
                    send_progress(format!(
                        "Scrubbing {}  {progress_percentage:02}%",
                        game.title()
                    ));

                    prev_percentage = progress_percentage;
                }

                Ok(())
            },
            &process_opts,
        )?;

        let out_writer = out_writer
            .into_inner()
            .map_err(|_| anyhow!("Failed to flush BufWriter"))?;

        if !finalization.header.is_empty() {
            out_writer.finalize(Some(&finalization.header))?;
        }
    }

    fs::remove_dir_all(game.path())?;
    fs::rename(tmp_game_dir, game.path())?;

    Ok(format!("Scrubbed {}", game.title()))
}
