use crate::{
    games::{
        disc_reader::{SharedMultiFileReader, retrieve_multi_disc_files},
        game::Game,
        import::{SPLIT_SIZE, get_filename},
    },
    long_operation::LongOperationState,
    util::misc::OPTIMAL_THREADS,
};
use anyhow::{Context, Result, anyhow};
use nod::{
    common::Format,
    read::{DiscOptions, DiscReader},
    write::{DiscWriter, FormatOptions, ProcessOptions, ScrubLevel},
};
use smol::stream::Stream;
use split_write::SplitWriter;
use std::{
    ffi::OsStr,
    io::{BufWriter, Write},
};

fn perform_blocking(game: &Game, tx: &smol::channel::Sender<LongOperationState>) -> Result<()> {
    let disc_path = game.get_disc_path_blocking().context("disc not found")?;

    let game_dir_name = game
        .path()
        .file_name()
        .and_then(OsStr::to_str)
        .context("invalid path")?;

    let tmp_game_dir_name = format!("{game_dir_name} SCRUB");
    let tmp_game_dir = game.path().with_file_name(tmp_game_dir_name);
    std::fs::create_dir(&tmp_game_dir)?;

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
        let files = retrieve_multi_disc_files(&disc_path)?;

        let must_split = files.len() > 1;
        let split_size = must_split.then_some(SPLIT_SIZE);

        let disc_reader = DiscReader::new_stream(
            Box::new(SharedMultiFileReader::new(files)?),
            &DiscOptions {
                preloader_threads: OPTIMAL_THREADS.preloader,
                ..Default::default()
            },
        )?;

        let disc_writer = DiscWriter::new(disc_reader, &format_opts)?;

        let out_writer = SplitWriter::create(
            &tmp_game_dir,
            |part| {
                get_filename(
                    game.id(),
                    true,
                    part,
                    0,
                    Format::Wbfs,
                    Format::Iso,
                    must_split,
                )
            },
            split_size,
        )?;
        let mut out_writer = BufWriter::new(out_writer);

        let mut prev_percentage = 100;
        let finalization = disc_writer.process(
            |data, progress, total| {
                out_writer.write_all(&data)?;

                let progress_percentage = progress * 100 / total;
                if progress_percentage != prev_percentage {
                    let _ = tx.try_send(LongOperationState::Progress(format!(
                        "Scrubbing {}  {progress_percentage:02}%",
                        game.title()
                    )));

                    prev_percentage = progress_percentage;
                }

                Ok(())
            },
            &process_opts,
        )?;

        let mut out_writer = out_writer
            .into_inner()
            .map_err(|_| anyhow!("Failed to get inner writer"))?;

        if !finalization.header.is_empty() {
            out_writer.write_header(&finalization.header)?;
        }

        out_writer.flush()?;
    }

    std::fs::remove_dir_all(game.path())?;
    std::fs::rename(tmp_game_dir, game.path())?;

    Ok(())
}

pub fn scrub_game(game: Game) -> impl Stream<Item = LongOperationState> {
    let (tx, rx) = smol::channel::bounded(1);

    let _ = std::thread::spawn(move || {
        let exit = match perform_blocking(&game, &tx) {
            Ok(()) => LongOperationState::Finished(format!("Scrubbed {}", game.title())),
            Err(e) => LongOperationState::Errored(e.to_string()),
        };

        tx.send_blocking(exit).expect("Channel should be open");
    });

    rx
}
