use crate::{
    games::{
        disc_reader::{SharedMultiFileReader, retrieve_multi_disc_files},
        game::Game,
        import::{SPLIT_SIZE, get_filename},
    },
    tasks::{self, Reporter, TaskControl, WorkerEvent},
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
    path::Path,
};

/// Writes the scrubbed game next to the game's folder and swaps the two once it is complete; on
/// failure or cancel only the unfinished copy is removed, the game stays as it was.
fn perform_blocking(game: &Game, control: &TaskControl, reporter: &mut Reporter) -> Result<()> {
    let disc_path = game.get_disc_path_blocking().context("disc not found")?;

    let game_dir_name = game
        .path()
        .file_name()
        .and_then(OsStr::to_str)
        .context("invalid path")?;

    let tmp_game_dir_name = format!("{game_dir_name} SCRUB");
    let tmp_game_dir = game.path().with_file_name(tmp_game_dir_name);
    std::fs::create_dir(&tmp_game_dir)?;

    if let Err(e) = write_scrubbed(game, &disc_path, &tmp_game_dir, control, reporter) {
        let _ = std::fs::remove_dir_all(&tmp_game_dir);
        return Err(e);
    }

    std::fs::remove_dir_all(game.path())?;
    std::fs::rename(tmp_game_dir, game.path())?;

    Ok(())
}

fn write_scrubbed(
    game: &Game,
    disc_path: &Path,
    tmp_game_dir: &Path,
    control: &TaskControl,
    reporter: &mut Reporter,
) -> Result<()> {
    reporter.progress(0, None, || format!("Scrubbing {}", game.title()));

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
        let files = retrieve_multi_disc_files(disc_path)?;

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
            tmp_game_dir,
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

        let finalization = disc_writer.process(
            |data, progress, total| {
                control.checkpoint().map_err(std::io::Error::other)?;
                out_writer.write_all(&data)?;
                reporter.progress(progress, Some(total), || {
                    format!("Scrubbing {}", game.title())
                });
                Ok(())
            },
            &process_opts,
        );

        // nod wraps the error; the task manager knows it was cancelled anyway
        control.checkpoint()?;
        let finalization = finalization?;

        let mut out_writer = out_writer
            .into_inner()
            .map_err(|_| anyhow!("Failed to get inner writer"))?;

        if !finalization.header.is_empty() {
            out_writer.write_header(&finalization.header)?;
        }

        out_writer.flush()?;
    }

    Ok(())
}

/// Removes the update partition of a Wii game on the drive as a task.
pub fn scrub_game(game: Game, control: TaskControl) -> impl Stream<Item = WorkerEvent> {
    tasks::spawn_blocking(move |reporter| {
        perform_blocking(&game, &control, reporter)
            .map(|()| format!("Scrubbed {}", game.title()))
            .map_err(|e| e.to_string())
    })
}
