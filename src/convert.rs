use crate::game::{CalculatedHashes, Game, VerificationStatus};
use crate::messages::BackgroundMessage;
use crate::util::redump;
use anyhow::Result;
use nod::read::{DiscOptions, DiscReader, PartitionEncryption};
use nod::write::{DiscWriter, FormatOptions, ProcessOptions};
use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Verify a game's integrity by processing the entire disc
pub fn verify_game(
    game: Box<Game>,
    sender: egui_inbox::UiInboxSender<BackgroundMessage>,
    cancelled: Arc<AtomicBool>,
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
            // Check for cancellation
            if cancelled.load(Ordering::Relaxed) {
                return Err(io::Error::new(
                    io::ErrorKind::Interrupted,
                    "Operation cancelled by user",
                ));
            }
            // Send progress updates
            let _ = sender.send(BackgroundMessage::VerificationProgress(pos, total));
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

    // Check against Redump database
    if let Some(crc32) = finalization.crc32 {
        if let Some(redump_entry) = redump::find_by_crc32(crc32) {
            // Check if SHA1 also matches
            if finalization
                .sha1
                .is_some_and(|sha| sha == redump_entry.sha1)
            {
                Ok(VerificationStatus::FullyVerified(redump_entry, calculated))
            } else {
                Ok(VerificationStatus::Failed(
                    format!(
                        "Partial match: {} (CRC32 matches, file differs - likely NKit v1)",
                        redump_entry.name
                    ),
                    Some(calculated),
                ))
            }
        } else {
            Ok(VerificationStatus::Failed(
                "Not in Redump database".to_string(),
                Some(calculated),
            ))
        }
    } else {
        Ok(VerificationStatus::Failed(
            "Failed to calculate hashes".to_string(),
            None,
        ))
    }
}
