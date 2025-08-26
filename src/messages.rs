use crate::app::{App, ConversionState, VerificationState};
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
    /// Signal for current file conversion progress
    ConversionProgress(u64, u64),
    /// Signal that a single file conversion has completed
    FileConverted,
    /// Signal that the conversion has completed (with result)
    ConversionComplete,
    /// Signal that the directory has changed
    DirectoryChanged,
    /// Signal that an error occurred
    Error(Error),
    /// Signal that an update is available
    UpdateCheckComplete(Option<UpdateInfo>),
    /// Signal to start verification of a game
    StartVerification(Box<Game>),
    /// Signal for current verification progress
    VerificationProgress(u64, u64),
    /// Signal that verification has completed (with result)
    VerificationComplete(PathBuf, Result<VerificationStatus, Error>),
    /// Signal to cancel ongoing operation (conversion or verification)
    CancelOperation,
}

/// Processes messages received from background tasks
pub fn handle_messages(app: &mut App, ctx: &egui::Context) {
    let sender = app.inbox.sender();

    for msg in app.inbox.read(ctx) {
        match msg {
            BackgroundMessage::ConversionProgress(progress, total) => {
                if let ConversionState::Converting {
                    total_files,
                    files_converted,
                    ..
                } = app.conversion_state
                {
                    app.conversion_state = ConversionState::Converting {
                        total_files,
                        files_converted,
                        current_progress: (progress, total),
                    };
                }
            }

            BackgroundMessage::FileConverted => {
                if let ConversionState::Converting {
                    total_files,
                    files_converted,
                    ..
                } = app.conversion_state
                {
                    app.conversion_state = ConversionState::Converting {
                        total_files,
                        files_converted: files_converted + 1,
                        current_progress: (0, 0),
                    };
                }
            }

            BackgroundMessage::ConversionComplete => {
                // Check if this was due to cancellation
                let was_cancelled = app.operation_cancelled.load(Ordering::Relaxed);
                app.conversion_state = ConversionState::Idle;

                if was_cancelled {
                    app.bottom_right_toasts.warning("Conversion cancelled");
                    // Reset the flag for next operation
                    app.operation_cancelled.store(false, Ordering::Relaxed);
                }
            }

            BackgroundMessage::DirectoryChanged => {
                if let Err(e) = app.refresh_games() {
                    let _ = sender.send(BackgroundMessage::Error(e));
                }
            }

            BackgroundMessage::Error(e) => {
                error!("{e:?}");
                let text = egui::RichText::new(e.to_string()).strong().size(16.0);
                app.bottom_right_toasts
                    .error(text)
                    .closable(true)
                    .duration(None);
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

            BackgroundMessage::StartVerification(game) => {
                app.verification_state = VerificationState::Verifying {
                    game_path: game.path.clone(),
                    progress: (0, 0),
                    queue: Vec::new(),
                    total_games: 1,
                    games_verified: 0,
                    games_passed: 0,
                    games_failed: 0,
                };
                // Spawn verification task
                app.spawn_verification(game);
            }

            BackgroundMessage::VerificationProgress(progress, total) => {
                if let VerificationState::Verifying {
                    game_path,
                    queue,
                    total_games,
                    games_verified,
                    games_passed,
                    games_failed,
                    ..
                } = app.verification_state.clone()
                {
                    app.verification_state = VerificationState::Verifying {
                        game_path,
                        progress: (progress, total),
                        queue,
                        total_games,
                        games_verified,
                        games_passed,
                        games_failed,
                    };
                }
            }

            BackgroundMessage::VerificationComplete(game_path, result) => {
                // Check if this was a cancellation by checking for interrupted IO error
                let was_cancelled = result
                    .as_ref()
                    .err()
                    .and_then(|e| e.downcast_ref::<nod::Error>())
                    .is_some_and(|nod_err| {
                        if let nod::Error::Io(_, io_err) = nod_err {
                            io_err.kind() == io::ErrorKind::Interrupted
                        } else {
                            false
                        }
                    });

                if was_cancelled {
                    // Cancelled - stop everything and return to idle
                    app.verification_state = VerificationState::Idle;
                    app.bottom_right_toasts.warning("Verification cancelled");
                    return;
                }

                // Track whether this game passed or failed
                let mut game_passed = false;

                // Handle the verification result
                if let Some(game) = app.games.iter_mut().find(|g| g.path == game_path) {
                    match result {
                        Ok(status) => {
                            // Check if this is a successful verification
                            game_passed = matches!(
                                status,
                                VerificationStatus::FullyVerified(_, _)
                                    | VerificationStatus::EmbeddedMatch(_)
                            );
                            game.set_verification_status(status);
                        }
                        Err(e) => {
                            let _ = sender.send(BackgroundMessage::Error(e));
                        }
                    }
                }

                // Check if there are more games in the queue
                if let VerificationState::Verifying {
                    queue,
                    total_games,
                    games_verified,
                    games_passed,
                    games_failed,
                    ..
                } = app.verification_state.clone()
                {
                    // Update counters based on this game's result
                    let new_games_passed = if game_passed {
                        games_passed + 1
                    } else {
                        games_passed
                    };
                    let new_games_failed = if !game_passed {
                        games_failed + 1
                    } else {
                        games_failed
                    };

                    if !queue.is_empty() {
                        // Continue with next game
                        let next_path = &queue[0];
                        if let Some(next_game) = app
                            .games
                            .iter()
                            .find(|g| &g.path == next_path)
                            .map(|g| Box::new(g.clone()))
                        {
                            let mut remaining_queue = queue.clone();
                            remaining_queue.remove(0);

                            app.verification_state = VerificationState::Verifying {
                                game_path: next_path.clone(),
                                progress: (0, 0),
                                queue: remaining_queue,
                                total_games,
                                games_verified: games_verified + 1,
                                games_passed: new_games_passed,
                                games_failed: new_games_failed,
                            };

                            app.spawn_verification(next_game);
                        }
                    } else {
                        // All done - show appropriate toast based on results
                        app.verification_state = VerificationState::Idle;

                        if new_games_failed > 0 {
                            // Some games failed - show warning
                            app.bottom_right_toasts.warning(format!(
                                "Verification complete: {} game{} passed, {} game{} failed",
                                new_games_passed,
                                if new_games_passed == 1 { "" } else { "s" },
                                new_games_failed,
                                if new_games_failed == 1 { "" } else { "s" }
                            ));
                        } else {
                            // All games passed - show success
                            app.bottom_right_toasts.success(format!(
                                "Verification complete! {} game{} verified",
                                total_games,
                                if total_games == 1 { "" } else { "s" }
                            ));
                        }
                    }
                } else {
                    // Single game verification
                    app.verification_state = VerificationState::Idle;
                    if game_passed {
                        app.bottom_right_toasts.success("Verification complete!");
                    } else {
                        app.bottom_right_toasts.warning("Verification failed!");
                    }
                }
            }

            BackgroundMessage::CancelOperation => {
                // Set cancellation flag - the thread will detect this and exit
                app.operation_cancelled.store(true, Ordering::Relaxed);
                // The actual "cancelled" message will be shown when the thread reports completion
            }
        }
    }
}
