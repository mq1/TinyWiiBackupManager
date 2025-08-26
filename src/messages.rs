use crate::app::{App, OperationResult, OperationState, OperationType};
use crate::cover_manager::CoverType;
use crate::game::{Game, VerificationStatus};
use crate::update_check::UpdateInfo;
use anyhow::Error;
use eframe::egui;
use log::error;
use std::io;
use std::path::PathBuf;
use std::sync::atomic::Ordering;

/// Messages that can be sent from background tasks to the main thread
#[derive(Debug)]
pub enum BackgroundMessage {
    /// Signal that we're starting to process an item (with name for display)
    OperationStartItem(usize, String),
    /// Signal for current operation progress (used for both conversion and verification)
    OperationProgress(u64, u64),
    /// Signal that an operation has completed for one item
    OperationItemComplete(PathBuf, OperationResult),
    /// Signal that the entire operation has completed
    OperationComplete,
    /// Signal that the directory has changed
    DirectoryChanged,
    /// Signal that an error occurred
    Error(Error),
    /// Signal that an update is available
    UpdateCheckComplete(Option<UpdateInfo>),
    /// Signal to start verification of a single game
    StartSingleVerification(Box<Game>),
    /// Signal to cancel ongoing operation
    CancelOperation,
    /// Signal that a cover has been downloaded
    CoverDownloaded {
        game_id: String,
        cover_type: CoverType,
        path: PathBuf,
    },
    /// Signal that a cover download failed
    CoverDownloadFailed {
        game_id: String,
        cover_type: CoverType,
        error: String,
    },
}

/// Processes messages received from background tasks
pub fn handle_messages(app: &mut App, ctx: &egui::Context) {
    let sender = app.inbox.sender();

    for msg in app.inbox.read(ctx) {
        match msg {
            BackgroundMessage::OperationProgress(progress, total) => {
                if let OperationState::InProgress {
                    operation,
                    total_items,
                    items_completed,
                    current_item,
                    items_passed,
                    items_failed,
                    ..
                } = app.operation_state.clone()
                {
                    app.operation_state = OperationState::InProgress {
                        operation,
                        total_items,
                        items_completed,
                        current_item,
                        current_progress: (progress, total),
                        items_passed,
                        items_failed,
                    };
                }
            }

            BackgroundMessage::OperationStartItem(i, name) => {
                if let OperationState::InProgress {
                    operation,
                    total_items,
                    items_passed,
                    items_failed,
                    ..
                } = app.operation_state.clone()
                {
                    app.operation_state = OperationState::InProgress {
                        operation,
                        total_items,
                        items_completed: i,
                        current_item: name,
                        current_progress: (0, 0),
                        items_passed,
                        items_failed,
                    };
                }
            }

            BackgroundMessage::OperationItemComplete(game_dir, result) => {
                let item_passed = match result {
                    OperationResult::ConversionComplete(calculated_hashes) => {
                        // Find the newly created game and update its verification status
                        if let Err(e) = app.refresh_games() {
                            let _ = sender.send(BackgroundMessage::Error(e));
                        }
                        if let Some(game) = app.games.iter_mut().find(|g| g.path == game_dir) {
                            game.set_verification_status(
                                calculated_hashes.into_verification_status(),
                            );
                            true
                        } else {
                            false
                        }
                    }
                    OperationResult::VerificationComplete(verification_status) => {
                        // Track whether this game passed or failed
                        let game_passed = matches!(
                            verification_status,
                            VerificationStatus::FullyVerified(_, _)
                                | VerificationStatus::EmbeddedMatch(_)
                        );
                        if let Some(game) = app.games.iter_mut().find(|g| g.path == game_dir) {
                            game.set_verification_status(verification_status);
                        }
                        game_passed
                    }
                    OperationResult::Error(e) => {
                        // Check if this was a cancellation
                        if let Some(nod::Error::Io(_, io_err)) = e.downcast_ref::<nod::Error>()
                            && io_err.kind() == io::ErrorKind::Interrupted
                        {
                            // Cancelled - continue processing messages, wait for OperationComplete
                            continue;
                        } else {
                            let _ = sender.send(BackgroundMessage::Error(e));
                            false
                        }
                    }
                };

                if let OperationState::InProgress {
                    operation,
                    total_items,
                    items_completed,
                    items_passed,
                    items_failed,
                    current_item,
                    current_progress,
                    ..
                } = app.operation_state.clone()
                {
                    app.operation_state = OperationState::InProgress {
                        operation,
                        total_items,
                        items_completed,
                        current_item,
                        current_progress,
                        items_passed: if item_passed {
                            items_passed + 1
                        } else {
                            items_passed
                        },
                        items_failed: if item_passed {
                            items_failed
                        } else {
                            items_failed + 1
                        },
                    };
                }
            }

            BackgroundMessage::OperationComplete => {
                // Check if this was due to cancellation
                let was_cancelled = app.operation_cancelled.load(Ordering::Relaxed);

                // Get operation type and stats before resetting state
                let (operation_type, total_items, items_passed, items_failed) =
                    if let OperationState::InProgress {
                        operation,
                        total_items,
                        items_passed,
                        items_failed,
                        ..
                    } = app.operation_state
                    {
                        (Some(operation), total_items, items_passed, items_failed)
                    } else {
                        (None, 0, 0, 0)
                    };

                app.operation_state = OperationState::Idle;

                if was_cancelled {
                    app.bottom_right_toasts.warning(match operation_type {
                        Some(OperationType::Converting) => "Conversion cancelled",
                        Some(OperationType::Verifying) => "Verification cancelled",
                        None => "Operation cancelled",
                    });
                    // Reset the flag for next operation
                    app.operation_cancelled.store(false, Ordering::Relaxed);
                } else if let Some(operation_type) = operation_type {
                    // Show completion message based on operation type
                    match operation_type {
                        OperationType::Converting => {
                            app.bottom_right_toasts.success(format!(
                                "Conversion complete! {} file{} converted",
                                total_items,
                                if total_items == 1 { "" } else { "s" }
                            ));
                        }
                        OperationType::Verifying => {
                            if items_failed > 0 {
                                app.bottom_right_toasts.warning(format!(
                                    "Verification complete: {} game{} passed, {} game{} failed",
                                    items_passed,
                                    if items_passed == 1 { "" } else { "s" },
                                    items_failed,
                                    if items_failed == 1 { "" } else { "s" }
                                ));
                            } else {
                                app.bottom_right_toasts.success(format!(
                                    "Verification complete! {} game{} verified",
                                    total_items,
                                    if total_items == 1 { "" } else { "s" }
                                ));
                            }
                        }
                    }
                }
            }

            BackgroundMessage::DirectoryChanged => {
                if let Err(e) = app.refresh_games() {
                    let _ = sender.send(BackgroundMessage::Error(e));
                }
            }

            BackgroundMessage::Error(e) => {
                error!("{e:?}");
                app.bottom_right_toasts.error(e.to_string());
            }

            BackgroundMessage::UpdateCheckComplete(update_info) => {
                if let Some(update_info) = update_info {
                    let update_text = format!("✨Update available: {}✨    ", update_info.version);

                    app.bottom_left_toasts.custom(
                        update_text,
                        "⬇".to_string(),
                        egui::Color32::GRAY,
                    );

                    app.update_info = Some(update_info);
                }
            }

            BackgroundMessage::StartSingleVerification(game) => {
                // Start verification of a single game
                app.spawn_verification(vec![game]);
            }

            BackgroundMessage::CancelOperation => {
                // Set cancellation flag - the thread will detect this and exit
                app.operation_cancelled.store(true, Ordering::Relaxed);
                // The actual "cancelled" message will be shown when the thread reports completion
            }

            BackgroundMessage::CoverDownloaded { game_id, cover_type, path: _ } => {
                // Cover downloaded successfully - UI will automatically refresh and show the cover
                log::debug!("Cover downloaded for {} ({:?})", game_id, cover_type);
            }

            BackgroundMessage::CoverDownloadFailed { game_id, cover_type, error } => {
                // Cover download failed - log the error but don't show toast (too noisy)
                log::debug!("Cover download failed for {} ({:?}): {}", game_id, cover_type, error);
            }
        }
    }
}
