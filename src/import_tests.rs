// SPDX-FileCopyrightText: 2026 TinyWiiBackupManager contributors
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::message::ConfigMessage,
    games::{
        ImportEntry,
        import::{
            BatchImportEvent, DEFAULT_PARALLEL_IMPORTS, ImportResult, ImportSummary,
            MAX_PARALLEL_IMPORTS,
        },
    },
    long_operation::{LongOperationKind, LongOperationState},
    message::Message,
    state::AppState,
    util::drive_state::DriveState,
};
use std::path::Path;
use tempfile::TempDir;

fn app() -> (TempDir, AppState) {
    let dir = tempfile::tempdir().unwrap();
    let data_dir: &'static Path = Box::leak(dir.path().to_path_buf().into_boxed_path());
    let (mut state, task) = AppState::boot(data_dir)();
    drop(task);
    drop(state.config.set_mount_point(dir.path().to_path_buf()));
    state.drive = smol::block_on(DriveState::load(dir.path().to_path_buf()));
    (dir, state)
}

fn entry(dir: &Path, filename: &str, id: &[u8; 6], disc: u8) -> ImportEntry {
    let mut data = vec![0; 0x50000];
    data[..6].copy_from_slice(id);
    data[6] = disc;
    data[0x1c..0x20].copy_from_slice(&[0xc2, 0x33, 0x9f, 0x3d]);
    data[0x20..0x24].copy_from_slice(b"Test");
    let path = dir.join(filename);
    std::fs::write(&path, data).unwrap();
    smol::block_on(ImportEntry::new(path)).unwrap()
}

fn progress(state: &AppState) -> &str {
    state
        .long_operations
        .iter()
        .find_map(|(operation, kind)| match (kind, operation) {
            (LongOperationKind::Import, LongOperationState::Progress(progress)) => {
                Some(progress.as_str())
            }
            _ => None,
        })
        .unwrap()
}

fn result(state: &mut AppState, run: u64, entry: ImportEntry, error: Option<&str>) {
    drop(state.update(Message::ImportBatchUpdated(
        run,
        BatchImportEvent::Result(ImportResult {
            entry,
            error: error.map(str::to_owned),
            warning: None,
            cancelled: false,
        }),
    )));
}

fn finish(state: &mut AppState, run: u64, imported: usize, failed: usize) {
    drop(state.update(Message::ImportBatchUpdated(
        run,
        BatchImportEvent::Finished(ImportSummary {
            imported,
            failed,
            cancelled: 0,
        }),
    )));
}

#[test]
fn import_selection_and_parallelism_are_explicit_and_bounded() {
    let (dir, mut state) = app();
    let first = entry(dir.path(), "a.iso", b"GAAE01", 0);
    let second = entry(dir.path(), "b.iso", b"GABE01", 0);
    state.add_to_import_queue(vec![first.clone(), second]);
    assert_eq!(state.import_parallel_jobs, DEFAULT_PARALLEL_IMPORTS);
    assert_eq!(DEFAULT_PARALLEL_IMPORTS, 2);
    assert_eq!(state.selected_import_count(), 2);
    drop(state.update(Message::SetImportParallelJobs(0)));
    assert_eq!(state.import_parallel_jobs, 1);
    drop(state.update(Message::SetImportParallelJobs(usize::MAX)));
    assert_eq!(state.import_parallel_jobs, MAX_PARALLEL_IMPORTS);

    drop(state.update(Message::SelectAllImportGames(false)));
    assert!(state.begin_import_run().is_none());
    assert!(!state.is_import_busy());
    let id = state.import_queue[0].id;
    drop(state.update(Message::ToggleImportGame(id, true)));
    let (_, selected) = state.begin_import_run().unwrap();
    assert_eq!(selected, vec![first]);
    assert!(state.is_import_busy());
}

#[test]
fn delayed_queue_actions_use_stable_ids_and_cannot_remove_another_game() {
    let (dir, mut state) = app();
    let first = entry(dir.path(), "a.iso", b"GAAE01", 0);
    let second = entry(dir.path(), "b.iso", b"GABE01", 0);
    state.add_to_import_queue(vec![first, second.clone()]);
    let removed_id = state.import_queue[0].id;
    drop(state.update(Message::CancelImport(removed_id)));
    drop(state.update(Message::CancelImport(removed_id)));
    drop(state.update(Message::CancelImport(u64::MAX)));
    drop(state.update(Message::ToggleImportGame(removed_id, false)));
    assert_eq!(state.import_queue.len(), 1);
    assert_eq!(state.import_queue[0].entry, second);
    assert!(state.import_queue[0].selected);
}

#[test]
fn queue_deduplicates_actual_game_and_disc_even_when_filenames_disagree() {
    let (dir, mut state) = app();
    let first_disc = entry(dir.path(), "first.iso", b"GAAE01", 0);
    let renamed_copy = entry(dir.path(), "fake (Disc 2).iso", b"GAAE01", 0);
    let second_disc = entry(dir.path(), "unlabelled.iso", b"GAAE01", 1);
    state.add_to_import_queue(vec![first_disc, renamed_copy, second_disc]);
    assert_eq!(state.import_queue.len(), 2);
    let (run, imports) = state.begin_import_run().unwrap();
    result(&mut state, run, imports[0].clone(), None);
    assert_eq!(state.import_queue.len(), 1);
    state.add_to_import_queue(imports);
    // Successful entries remain reserved until the whole batch finishes.
    assert_eq!(state.import_queue.len(), 1);
}

#[test]
fn batch_is_frozen_and_newly_added_games_wait_for_the_next_start() {
    let (dir, mut state) = app();
    let first = entry(dir.path(), "a.iso", b"GAAE01", 0);
    let second = entry(dir.path(), "b.iso", b"GABE01", 0);
    let later = entry(dir.path(), "c.iso", b"GACE01", 0);
    state.add_to_import_queue(vec![first.clone(), second.clone()]);
    let first_id = state.import_queue[0].id;
    let (run, imports) = state.begin_import_run().unwrap();
    assert_eq!(imports, vec![first.clone(), second.clone()]);
    state.add_to_import_queue(vec![later.clone()]);
    drop(state.update(Message::CancelImport(first_id)));
    drop(state.update(Message::CancelAllImports));
    drop(state.update(Message::SelectAllImportGames(false)));
    drop(state.update(Message::ToggleImportGame(first_id, false)));
    drop(state.update(Message::SetImportParallelJobs(4)));
    assert!(state.begin_import_run().is_none());
    assert_eq!(state.import_queue.len(), 3);
    assert_eq!(state.selected_import_count(), 3);
    assert_eq!(state.import_parallel_jobs, 2);
    assert_eq!(state.active_import.as_ref().unwrap().entries.len(), 2);

    result(&mut state, run, first, None);
    result(&mut state, run, second, None);
    assert!(state.is_import_busy());
    finish(&mut state, run, 2, 0);
    assert!(!state.is_import_busy());
    assert_eq!(state.import_queue.len(), 1);
    assert_eq!(state.import_queue[0].entry, later);
}

#[test]
fn import_results_remove_successes_keep_failures_and_ignore_duplicate_results() {
    let (dir, mut state) = app();
    let first = entry(dir.path(), "a.iso", b"GAAE01", 0);
    let second = entry(dir.path(), "b.iso", b"GABE01", 0);
    state.add_to_import_queue(vec![first.clone(), second.clone()]);
    let (run, _) = state.begin_import_run().unwrap();
    result(&mut state, run, first.clone(), None);
    result(&mut state, run, first, Some("duplicate event"));
    result(&mut state, run, second.clone(), Some("drive full"));
    result(&mut state, run, second, None);
    assert_eq!(state.import_queue.len(), 1);
    assert_eq!(state.import_queue[0].error.as_deref(), Some("drive full"));
    assert_eq!(state.notifications.iter().count(), 1);
    assert!(state.is_import_busy());
    finish(&mut state, run, 1, 1);
    assert!(!state.is_import_busy());
    assert_eq!(state.import_queue.len(), 1);
    assert!(state.import_queue[0].selected);

    state.drive = smol::block_on(DriveState::load(dir.path().to_path_buf()));
    assert!(state.begin_import_run().is_some());
    assert!(state.import_queue[0].error.is_none());
}

#[test]
fn stale_or_unscoped_import_events_cannot_unlock_or_modify_an_active_batch() {
    let (dir, mut state) = app();
    let first = entry(dir.path(), "a.iso", b"GAAE01", 0);
    state.add_to_import_queue(vec![first.clone()]);
    let (old, _) = state.begin_import_run().unwrap();
    finish(&mut state, old, 0, 1);
    state.drive = smol::block_on(DriveState::load(dir.path().to_path_buf()));
    let (new, _) = state.begin_import_run().unwrap();
    assert_ne!(old, new);
    drop(state.update(Message::ImportBatchUpdated(
        new,
        BatchImportEvent::Progress("Active batch".into()),
    )));
    result(&mut state, old, first, None);
    finish(&mut state, old, 1, 0);
    drop(state.update(Message::ImportBatchUpdated(
        old,
        BatchImportEvent::Progress("Old batch".into()),
    )));
    drop(state.update(Message::SetLongOperationState(
        LongOperationKind::Import,
        LongOperationState::Finished("Old unscoped task".into()),
    )));
    assert_eq!(progress(&state), "Active batch");
    assert_eq!(state.active_import.as_ref().unwrap().id, new);
    assert_eq!(state.import_queue.len(), 1);
    assert_eq!(state.notifications.iter().count(), 1);
    finish(&mut state, new, 0, 1);
    finish(&mut state, new, 0, 1);
    assert_eq!(state.notifications.iter().count(), 2);
}

#[test]
fn changing_drive_or_closing_modal_keeps_the_import_run_and_original_destination() {
    let (dir, mut state) = app();
    state.add_to_import_queue(vec![entry(dir.path(), "a.iso", b"GAAE01", 0)]);
    let (run, _) = state.begin_import_run().unwrap();
    let other = tempfile::tempdir().unwrap();
    drop(state.update(Message::CloseModal));
    drop(state.update(Message::MountPointPicked(other.path().to_path_buf())));
    assert!(state.is_import_busy());
    assert_eq!(state.active_import.as_ref().unwrap().id, run);
    assert_eq!(
        state.active_import.as_ref().unwrap().mount_point,
        dir.path()
    );
    assert_eq!(state.config.mount_point(), other.path());
    finish(&mut state, run, 1, 0);
    assert!(!state.is_import_busy());
    assert_eq!(state.config.mount_point(), other.path());
}

#[test]
fn a_post_commit_warning_keeps_the_import_successful_and_does_not_offer_a_retry() {
    let (dir, mut state) = app();
    let imported = entry(dir.path(), "a.iso", b"GAAE01", 0);
    state.add_to_import_queue(vec![imported.clone()]);
    let (run, _) = state.begin_import_run().unwrap();
    drop(state.update(Message::ImportBatchUpdated(
        run,
        BatchImportEvent::Result(ImportResult {
            entry: imported,
            error: None,
            warning: Some("Imported successfully, but the source could not be removed".into()),
            cancelled: false,
        }),
    )));
    assert!(state.import_queue.is_empty());
    assert!(state.is_import_busy());
    assert_eq!(
        state.notifications.iter().next().unwrap().level(),
        crate::notifications::notification::NotificationLevel::Warning
    );
    finish(&mut state, run, 1, 0);
    assert!(!state.is_import_busy());
}

fn cancelled_result(state: &mut AppState, run: u64, entry: ImportEntry, warning: Option<&str>) {
    drop(state.update(Message::ImportBatchUpdated(
        run,
        BatchImportEvent::Result(ImportResult {
            entry,
            error: None,
            warning: warning.map(str::to_owned),
            cancelled: true,
        }),
    )));
}

fn finish_cancelled(state: &mut AppState, run: u64, imported: usize, cancelled: usize) {
    drop(state.update(Message::ImportBatchUpdated(
        run,
        BatchImportEvent::Finished(ImportSummary {
            imported,
            failed: 0,
            cancelled,
        }),
    )));
}

#[test]
fn cancel_import_stays_busy_until_cleanup_finishes_and_preserves_remaining_entries() {
    for jobs in [1, 2, 4] {
        let (dir, mut state) = app();
        let first = entry(dir.path(), "a.iso", b"GAAE01", 0);
        let second = entry(dir.path(), "b.iso", b"GABE01", 0);
        let third = entry(dir.path(), "c.iso", b"GACE01", 0);
        state.add_to_import_queue(vec![first.clone(), second.clone(), third]);
        drop(state.update(Message::SetImportParallelJobs(jobs)));
        let (run, _) = state.begin_import_run().unwrap();
        let token = state.active_import.as_ref().unwrap().cancellation.clone();
        result(&mut state, run, first, None);
        drop(state.update(Message::CancelImportRun(run)));
        drop(state.update(Message::CancelImportRun(run)));
        assert!(token.is_cancelled());
        assert!(state.is_import_cancelling());
        assert!(state.is_import_busy());
        assert!(state.begin_import_run().is_none());
        drop(state.update(Message::CancelAllImports));
        drop(state.update(Message::ImportBatchUpdated(
            run,
            BatchImportEvent::Progress("Late progress".into()),
        )));
        assert_eq!(progress(&state), "Cancelling import...");
        cancelled_result(&mut state, run, second.clone(), None);
        // A duplicate completion must not remove an entry cancelled earlier.
        result(&mut state, run, second, None);
        assert!(state.is_import_busy());
        assert_eq!(state.import_queue.len(), 2);
        assert!(!state.notifications.has_notifications());
        finish_cancelled(&mut state, run, 1, 2);
        assert!(!state.is_import_busy());
        assert!(!state.is_import_cancelling());
        assert_eq!(state.selected_import_count(), 2);
        assert!(state.import_queue.iter().all(|row| row.error.is_none()));
        assert_eq!(
            state.notifications.iter().next().unwrap().level(),
            crate::notifications::notification::NotificationLevel::Info
        );
    }
}

#[test]
fn stale_cancel_and_results_cannot_stop_or_modify_a_retry() {
    let (dir, mut state) = app();
    let source = entry(dir.path(), "a.iso", b"GAAE01", 0);
    state.add_to_import_queue(vec![source.clone()]);
    let (old, _) = state.begin_import_run().unwrap();
    let old_token = state.active_import.as_ref().unwrap().cancellation.clone();
    drop(state.update(Message::CancelImportRun(old)));
    cancelled_result(&mut state, old, source.clone(), None);
    finish_cancelled(&mut state, old, 0, 1);
    drop(state.update(Message::CancelImportRun(old)));
    assert!(!state.is_import_busy());
    assert_eq!(state.notifications.iter().count(), 1);
    state.drive = smol::block_on(DriveState::load(dir.path().to_path_buf()));
    let (new, _) = state.begin_import_run().unwrap();
    let new_token = state.active_import.as_ref().unwrap().cancellation.clone();
    assert_ne!(old, new);
    assert!(old_token.is_cancelled());
    assert!(!new_token.is_cancelled());
    drop(state.update(Message::ImportBatchUpdated(
        new,
        BatchImportEvent::Progress("Retry".into()),
    )));
    drop(state.update(Message::CancelImportRun(old)));
    cancelled_result(&mut state, old, source.clone(), None);
    finish_cancelled(&mut state, old, 0, 1);
    assert_eq!(progress(&state), "Retry");
    assert!(!state.is_import_cancelling());
    assert_eq!(state.import_queue.len(), 1);
    result(&mut state, new, source, None);
    finish(&mut state, new, 1, 0);
    assert!(state.import_queue.is_empty());
}

#[test]
fn import_can_be_cancelled_after_navigation_modal_close_and_drive_change() {
    let (dir, mut state) = app();
    state.add_to_import_queue(vec![entry(dir.path(), "a.iso", b"GAAE01", 0)]);
    let (run, _) = state.begin_import_run().unwrap();
    drop(state.update(Message::NavigateTo(crate::ui::pages::Page::Settings)));
    drop(state.update(Message::CloseModal));
    let other = tempfile::tempdir().unwrap();
    drop(state.update(Message::MountPointPicked(other.path().to_path_buf())));
    drop(state.update(Message::CancelImportRun(run)));
    assert!(
        state
            .active_import
            .as_ref()
            .unwrap()
            .cancellation
            .is_cancelled()
    );
    assert_eq!(
        state.active_import.as_ref().unwrap().mount_point,
        dir.path()
    );
    assert_eq!(progress(&state), "Cancelling import...");
    assert!(state.is_import_busy());
    finish_cancelled(&mut state, run, 0, 1);
    assert!(!state.is_import_busy());
    assert_eq!(state.config.mount_point(), other.path());
}

#[test]
fn completed_import_wins_when_cancel_races_with_publication() {
    let (dir, mut state) = app();
    let source = entry(dir.path(), "a.iso", b"GAAE01", 0);
    state.add_to_import_queue(vec![source.clone()]);
    let (run, _) = state.begin_import_run().unwrap();
    drop(state.update(Message::CancelImportRun(run)));
    result(&mut state, run, source, None);
    assert!(state.import_queue.is_empty());
    assert!(state.is_import_busy());
    finish(&mut state, run, 1, 0);
    assert!(!state.is_import_busy());
    assert_eq!(
        state.notifications.iter().next().unwrap().level(),
        crate::notifications::notification::NotificationLevel::Success
    );
}

#[test]
fn cancellation_cleanup_warnings_remain_visible_without_removing_retry_entries() {
    let (dir, mut state) = app();
    let source = entry(dir.path(), "a.iso", b"GAAE01", 0);
    state.add_to_import_queue(vec![source.clone()]);
    let (run, _) = state.begin_import_run().unwrap();
    drop(state.update(Message::CancelImportRun(run)));
    cancelled_result(
        &mut state,
        run,
        source,
        Some("Could not remove an incomplete temporary file"),
    );
    assert_eq!(state.import_queue.len(), 1);
    assert!(state.import_queue[0].error.is_none());
    assert!(state.import_queue[0].selected);
    assert_eq!(
        state.notifications.iter().next().unwrap().level(),
        crate::notifications::notification::NotificationLevel::Warning
    );
    finish_cancelled(&mut state, run, 0, 1);
    assert_eq!(state.notifications.iter().count(), 2);
}

#[test]
fn a_real_import_failure_is_still_reported_after_a_cancellation_request() {
    let (dir, mut state) = app();
    let source = entry(dir.path(), "a.iso", b"GAAE01", 0);
    state.add_to_import_queue(vec![source.clone()]);
    let (run, _) = state.begin_import_run().unwrap();
    drop(state.update(Message::CancelImportRun(run)));
    result(&mut state, run, source, Some("Device disconnected"));
    assert_eq!(
        state.import_queue[0].error.as_deref(),
        Some("Device disconnected")
    );
    assert_eq!(
        state.notifications.iter().next().unwrap().level(),
        crate::notifications::notification::NotificationLevel::Error
    );
    finish(&mut state, run, 0, 1);
    assert!(!state.is_import_busy());
}

#[test]
fn fast_transfer_is_frozen_during_import_and_cancellation_without_changing_job_count() {
    let (dir, mut state) = app();
    state.add_to_import_queue(vec![entry(dir.path(), "a.iso", b"GAAE01", 0)]);
    assert!(state.can_change_fast_transfer());
    drop(state.update(Message::SetImportParallelJobs(4)));
    drop(state.update(Message::Config(ConfigMessage::SetFastTransfer(true))));
    assert!(state.config.fast_transfer());
    assert_eq!(state.import_parallel_jobs, 4);
    let (run, _) = state.begin_import_run().unwrap();
    assert!(!state.can_change_fast_transfer());
    drop(state.update(Message::Config(ConfigMessage::SetFastTransfer(false))));
    assert!(state.config.fast_transfer());
    let mut delayed_config = state.config.clone();
    drop(delayed_config.update(ConfigMessage::SetFastTransfer(false)));
    drop(state.update(Message::GotConfig(delayed_config)));
    assert!(state.config.fast_transfer());
    drop(state.update(Message::CancelImportRun(run)));
    drop(state.update(Message::Config(ConfigMessage::SetFastTransfer(false))));
    assert!(state.config.fast_transfer());
    assert!(!state.can_change_fast_transfer());
    finish_cancelled(&mut state, run, 0, 1);
    assert!(state.can_change_fast_transfer());
    drop(state.update(Message::Config(ConfigMessage::SetFastTransfer(false))));
    assert!(!state.config.fast_transfer());
    assert_eq!(state.import_parallel_jobs, 4);
}
