use super::{
    Job, JobContext, JobResult, JobState, should_cancel, start_job, update_status_with_bytes,
};
use crate::game::{CalculatedHashes, Game, VerificationStatus};
use anyhow::Result;
use nod::read::{DiscOptions, DiscReader, PartitionEncryption};
use nod::write::{DiscWriter, FormatOptions, ProcessOptions};
use std::io;
use std::sync::mpsc::Receiver;
use std::task::Waker;

pub struct VerifyConfig {
    pub games: Vec<Box<Game>>,
}

pub struct VerifyResult {
    pub verified: usize,
    pub passed: usize,
    pub failed: usize,
}

pub fn start_verify(waker: Waker, config: VerifyConfig) -> JobState {
    let title = format!(
        "Verify {} game{}",
        config.games.len(),
        if config.games.len() == 1 { "" } else { "s" }
    );

    start_job(waker, &title, Job::Verify, move |context, cancel| {
        verify_games(context, cancel, config).map(JobResult::Verify)
    })
}

fn verify_games(
    context: JobContext,
    cancel: Receiver<()>,
    config: VerifyConfig,
) -> Result<Box<VerifyResult>> {
    let mut verified = 0;
    let mut passed = 0;
    let mut failed = 0;
    let mut was_cancelled = false;

    let total = config.games.len() as u32;

    for (idx, game) in config.games.into_iter().enumerate() {
        // Check for cancellation
        if should_cancel(&cancel) {
            was_cancelled = true;
            break;
        }

        let game_name = game.title.clone();

        // Perform the verification
        match verify_game(
            &game,
            idx as u32,
            total,
            game_name.clone(),
            &context,
            &cancel,
        ) {
            Ok(status) => {
                verified += 1;
                match status {
                    VerificationStatus::FullyVerified(_, _)
                    | VerificationStatus::EmbeddedMatch(_) => {
                        passed += 1;
                        log::info!("{} verification: PASSED", game_name);
                    }
                    VerificationStatus::Failed(_, _) => {
                        failed += 1;
                        log::warn!("{} verification: FAILED", game_name);
                    }
                    VerificationStatus::NotVerified => {
                        log::info!("{} verification: NOT VERIFIED", game_name);
                    }
                }
            }
            Err(e) => {
                // Check if it was cancelled
                if e.to_string().contains("Cancelled") {
                    was_cancelled = true;
                    break;
                }
                failed += 1;
                log::warn!("Failed to verify {}: {}", game_name, e);
            }
        }
    }

    // Final status
    let final_status = if was_cancelled {
        format!(
            "Verification cancelled ({} completed, {} passed, {} failed)",
            verified, passed, failed
        )
    } else {
        format!(
            "Verified {} game{} ({} passed, {} failed)",
            verified,
            if verified == 1 { "" } else { "s" },
            passed,
            failed
        )
    };

    let _ = update_status_with_bytes(&context, final_status, total, total, 1, 1, None, &cancel);

    Ok(Box::new(VerifyResult {
        verified,
        passed,
        failed,
    }))
}

/// Verify a single game's integrity by processing the entire disc
fn verify_game(
    game: &Game,
    current_idx: u32,
    total_items: u32,
    game_name: String,
    context: &JobContext,
    cancel: &Receiver<()>,
) -> Result<VerificationStatus> {
    let disc_path = game
        .get_disc_file_path()
        .ok_or_else(|| anyhow::anyhow!("No disc image found"))?;

    // Open the disc
    let disc = DiscReader::new(
        &disc_path,
        &DiscOptions {
            partition_encryption: PartitionEncryption::Original,
            preloader_threads: 1,
        },
    )?;
    let disc_writer = DiscWriter::new(disc, &FormatOptions::default())?;
    let total = disc_writer.progress_bound();

    // Process the disc to calculate hashes
    let finalization = disc_writer.process(
        |_data, pos, _| {
            // Send progress updates (this will check for cancellation)
            update_status_with_bytes(
                context,
                format!("Verifying {}", game_name),
                current_idx,
                total_items,
                pos,
                total,
                Some(game_name.clone()),
                cancel,
            )
            .map_err(|_| io::Error::new(io::ErrorKind::Interrupted, "Cancelled"))?;

            Ok(())
        },
        &ProcessOptions {
            processor_threads: 0,
            digest_crc32: true,
            digest_md5: false, // MD5 is slow, skip it
            digest_sha1: true,
            digest_xxh64: true,
        },
    )?;

    // Store calculated hashes
    let calculated = CalculatedHashes {
        crc32: finalization.crc32,
        sha1: finalization.sha1,
        xxh64: finalization.xxh64,
    };

    // Check against Redump database using the shared logic
    Ok(calculated.into_verification_status())
}
