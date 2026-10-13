// SPDX-FileCopyrightText: 2026 TinyWiiBackupManager contributors
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::SortBy,
    games::{
        export::{DEFAULT_PARALLEL_EXPORTS, ExportFormat, MAX_PARALLEL_EXPORTS},
        game::Game,
        games_state::GamesState,
    },
    long_operation::{LongOperationKind, LongOperationState},
    message::Message,
    notifications::notification::NotificationLevel,
    state::AppState,
    ui::modals::{Modal, export_games::ExportSelection},
};
use std::path::Path;
use tempfile::TempDir;

fn app_with_games() -> (TempDir, AppState) {
    let dir = tempfile::tempdir().expect("create temporary drive");
    for name in [
        "wbfs/First Wii copy [RMGE01]",
        "wbfs/Second Wii copy [RMGE01]",
        "games/GameCube [GALE01]",
    ] {
        std::fs::create_dir_all(dir.path().join(name)).expect("create game directory");
    }

    // Boot tasks stay unpolled: these tests do not start dialogs or network requests.
    let data_dir: &'static Path = Box::leak(dir.path().to_path_buf().into_boxed_path());
    let (mut state, boot_task) = AppState::boot(data_dir)();
    drop(boot_task);
    drop(state.config.set_mount_point(dir.path().to_path_buf()));
    state.games = smol::block_on(GamesState::load(dir.path().to_path_buf()));
    (dir, state)
}

fn all_games(state: &AppState) -> Vec<Game> {
    let GamesState::Loaded(games) = &state.games else {
        panic!("expected loaded games");
    };
    games.iter_all().cloned().collect()
}

fn selection(state: &AppState) -> &ExportSelection {
    let Some(Modal::ExportGames(selection)) = &state.current_modal else {
        panic!("expected export selection modal");
    };
    selection
}

fn open_and_select_all(state: &mut AppState) {
    drop(state.update(Message::OpenExportGames));
    drop(state.update(Message::SelectAllExportGames(true)));
}

#[test]
fn export_select_all_includes_games_hidden_by_main_page_filters() {
    let (_dir, mut state) = app_with_games();
    let expected = all_games(&state);
    assert_eq!(expected.len(), 3);

    drop(state.update(Message::SearchGames("no matching game".into())));
    drop(state.update(Message::ToggleShowWii(false)));
    drop(state.update(Message::ToggleShowNgc(false)));
    let GamesState::Loaded(games) = &state.games else {
        panic!("expected loaded games");
    };
    assert_eq!(games.iter_by(SortBy::NameAscending).count(), 0);

    open_and_select_all(&mut state);
    assert_eq!(selection(&state).selected_games(), expected);
}

#[test]
fn games_with_duplicate_ids_can_be_selected_independently() {
    let (_dir, mut state) = app_with_games();
    let games = all_games(&state);
    let duplicate_indices: Vec<_> = games
        .iter()
        .enumerate()
        .filter_map(|(index, game)| game.is_wii().then_some(index))
        .collect();
    assert_eq!(duplicate_indices.len(), 2);
    let (first, second) = (duplicate_indices[0], duplicate_indices[1]);
    assert_eq!(games[first].id(), games[second].id());
    assert_ne!(games[first].path(), games[second].path());

    drop(state.update(Message::OpenExportGames));
    drop(state.update(Message::SelectAllExportGames(false)));
    drop(state.update(Message::ToggleExportGame(first, true)));
    assert_eq!(
        selection(&state).selected_games(),
        vec![games[first].clone()]
    );
    drop(state.update(Message::ToggleExportGame(second, true)));
    drop(state.update(Message::ToggleExportGame(first, false)));
    assert_eq!(
        selection(&state).selected_games(),
        vec![games[second].clone()]
    );

    drop(state.update(Message::SelectAllExportGames(true)));
    assert_eq!(selection(&state).selected_count(), games.len());
    drop(state.update(Message::SelectAllExportGames(false)));
    assert!(selection(&state).selected_games().is_empty());
}

#[test]
fn empty_selection_cannot_open_a_destination_picker() {
    let (_dir, mut state) = app_with_games();
    drop(state.update(Message::OpenExportGames));
    drop(state.update(Message::SelectAllExportGames(false)));
    drop(state.update(Message::PickBatchExportDestination));

    assert!(state.export_picker.is_none());
    assert!(!state.is_export_busy());
    assert_eq!(selection(&state).selected_count(), 0);
}

#[test]
fn cancelling_destination_keeps_choices_and_freezes_them_while_pending() {
    let (_dir, mut state) = app_with_games();
    drop(state.update(Message::OpenExportGames));
    drop(state.update(Message::SelectAllExportGames(false)));
    drop(state.update(Message::ToggleExportGame(1, true)));
    drop(state.update(Message::SetExportFormat(ExportFormat::Iso)));
    drop(state.update(Message::SetExportParallelJobs(3)));
    let expected = selection(&state).selected_games();

    drop(state.update(Message::PickBatchExportDestination));
    let request = state.export_picker.expect("destination picker is pending");
    assert!(state.is_export_busy());
    drop(state.update(Message::ToggleExportGame(1, false)));
    drop(state.update(Message::SelectAllExportGames(true)));
    drop(state.update(Message::SetExportFormat(ExportFormat::Rvz)));
    drop(state.update(Message::SetExportParallelJobs(1)));
    drop(state.update(Message::PickBatchExportDestination));
    assert_eq!(state.export_picker, Some(request));
    assert_eq!(selection(&state).selected_games(), expected);
    assert_eq!(selection(&state).format, ExportFormat::Iso);
    assert_eq!(selection(&state).parallel_jobs, 3);

    drop(state.update(Message::BatchExportDestinationPicked(request, None)));
    assert!(!state.is_export_busy());
    assert_eq!(selection(&state).selected_games(), expected);
    assert_eq!(selection(&state).format, ExportFormat::Iso);
    assert_eq!(selection(&state).parallel_jobs, 3);

    drop(state.update(Message::SetExportParallelJobs(1)));
    assert_eq!(selection(&state).parallel_jobs, 1);
}

#[test]
fn simultaneous_exports_default_to_two_and_are_bounded() {
    let (_dir, mut state) = app_with_games();
    drop(state.update(Message::OpenExportGames));
    assert_eq!(selection(&state).parallel_jobs, DEFAULT_PARALLEL_EXPORTS);
    assert_eq!(DEFAULT_PARALLEL_EXPORTS, 2);

    drop(state.update(Message::SetExportParallelJobs(0)));
    assert_eq!(selection(&state).parallel_jobs, 1);
    drop(state.update(Message::SetExportParallelJobs(usize::MAX)));
    assert_eq!(selection(&state).parallel_jobs, MAX_PARALLEL_EXPORTS);
    drop(state.update(Message::SetExportParallelJobs(3)));
    assert_eq!(selection(&state).parallel_jobs, 3);
}

#[test]
fn closing_and_reopening_selection_rejects_the_old_picker_result() {
    let (dir, mut state) = app_with_games();
    open_and_select_all(&mut state);
    drop(state.update(Message::PickBatchExportDestination));
    let old_request = state.export_picker.expect("first picker is pending");
    drop(state.update(Message::CloseModal));
    assert!(state.current_modal.is_none());
    assert!(!state.is_export_busy());

    open_and_select_all(&mut state);
    drop(state.update(Message::PickBatchExportDestination));
    let new_request = state.export_picker.expect("second picker is pending");
    assert_ne!(old_request, new_request);
    let expected = selection(&state).selected_games();
    drop(state.update(Message::BatchExportDestinationPicked(
        old_request,
        Some(dir.path().to_path_buf()),
    )));
    assert_eq!(state.export_picker, Some(new_request));
    assert_eq!(selection(&state).selected_games(), expected);
    assert!(state.long_operations.is_idle(LongOperationKind::Export));

    drop(state.update(Message::BatchExportDestinationPicked(new_request, None)));
    assert!(!state.is_export_busy());
}

#[test]
fn changing_drive_invalidates_selection_and_pending_picker() {
    let (dir, mut state) = app_with_games();
    open_and_select_all(&mut state);
    drop(state.update(Message::PickBatchExportDestination));
    let request = state.export_picker.expect("destination picker is pending");
    let new_drive = tempfile::tempdir().expect("create second drive");
    drop(state.update(Message::MountPointPicked(new_drive.path().to_path_buf())));
    assert!(state.current_modal.is_none());
    assert!(!state.is_export_busy());

    drop(state.update(Message::BatchExportDestinationPicked(
        request,
        Some(dir.path().to_path_buf()),
    )));
    assert!(state.current_modal.is_none());
    assert!(!state.is_export_busy());
}

#[test]
fn single_game_picker_blocks_batch_and_rejects_duplicate_results() {
    let (dir, mut state) = app_with_games();
    let game = all_games(&state).remove(0);
    drop(state.update(Message::PickExportDest(game.clone())));
    let request = state.export_picker.expect("single game picker is pending");
    drop(state.update(Message::OpenExportGames));
    drop(state.update(Message::PickExportDest(game.clone())));
    assert!(state.current_modal.is_none());
    assert_eq!(state.export_picker, Some(request));

    drop(state.update(Message::ExportGame(request, game.clone(), None)));
    assert!(!state.is_export_busy());
    drop(state.update(Message::ExportGame(
        request,
        game,
        Some(dir.path().join("duplicate-result.iso")),
    )));
    assert!(!state.is_export_busy());
    assert!(!dir.path().join("duplicate-result.iso").exists());
}

#[test]
fn an_active_export_blocks_both_destination_pickers() {
    let (_dir, mut state) = app_with_games();
    let game = all_games(&state).remove(0);
    open_and_select_all(&mut state);
    state.long_operations.set(
        LongOperationKind::Export,
        LongOperationState::Progress("Exporting existing batch".into()),
    );

    drop(state.update(Message::PickBatchExportDestination));
    drop(state.update(Message::PickExportDest(game)));
    assert!(state.export_picker.is_none());
    assert!(state.is_export_busy());
}

#[test]
fn reloading_the_game_list_does_not_replace_an_open_selection() {
    let (_dir, mut state) = app_with_games();
    open_and_select_all(&mut state);
    drop(state.update(Message::ToggleExportGame(0, false)));
    let expected = selection(&state).selected_games();

    // A background drive refresh may replace the list while the modal is open.
    drop(state.update(Message::GotGames(GamesState::NotLoaded)));
    assert_eq!(selection(&state).selected_games(), expected);
    assert_eq!(selection(&state).selected_count(), 2);
}

fn export_progress(state: &AppState) -> &str {
    state
        .long_operations
        .iter()
        .find_map(|(operation, kind)| match (kind, operation) {
            (LongOperationKind::Export, LongOperationState::Progress(progress)) => {
                Some(progress.as_str())
            }
            _ => None,
        })
        .expect("export is running")
}

#[test]
fn cancel_invalidates_single_and_batch_destination_results() {
    for batch in [false, true] {
        let (dir, mut state) = app_with_games();
        let game = all_games(&state).remove(0);
        if batch {
            open_and_select_all(&mut state);
            drop(state.update(Message::PickBatchExportDestination));
        } else {
            drop(state.update(Message::PickExportDest(game.clone())));
        }
        let request = state.export_picker.expect("picker is pending");
        drop(state.update(Message::CancelExport));
        drop(state.update(Message::CancelExport));
        assert!(!state.is_export_busy());
        assert_eq!(state.notifications.iter().count(), 1);

        let late_result = if batch {
            Message::BatchExportDestinationPicked(request, Some(dir.path().to_path_buf()))
        } else {
            Message::ExportGame(request, game, Some(dir.path().join("cancelled.iso")))
        };
        drop(state.update(late_result));
        assert!(!state.is_export_busy());
        assert!(state.active_export.is_none());
        assert!(!dir.path().join("cancelled.iso").exists());
    }
}

#[test]
fn single_sequential_and_parallel_starts_create_cancellable_runs() {
    for jobs in [None, Some(1), Some(3)] {
        let (dir, mut state) = app_with_games();
        if let Some(jobs) = jobs {
            open_and_select_all(&mut state);
            drop(state.update(Message::SetExportParallelJobs(jobs)));
            drop(state.update(Message::PickBatchExportDestination));
            let request = state.export_picker.expect("picker is pending");
            drop(state.update(Message::BatchExportDestinationPicked(
                request,
                Some(dir.path().to_path_buf()),
            )));
        } else {
            let game = all_games(&state).remove(0);
            drop(state.update(Message::PickExportDest(game.clone())));
            let request = state.export_picker.expect("picker is pending");
            drop(state.update(Message::ExportGame(
                request,
                game,
                Some(dir.path().join("single.iso")),
            )));
        }

        // These synthetic game directories contain no images. The task is dropped,
        // and only explicitly delivered terminal events are allowed to clear the UI.
        let run = state.active_export.clone().expect("active export token");
        drop(state.update(Message::CancelExport));
        assert!(run.cancellation.is_cancelled());
        assert!(state.is_export_busy());
        assert!(state.is_export_cancelling());
        assert_eq!(export_progress(&state), "Cancelling export...");
        drop(state.update(Message::ExportProgress(
            run.id,
            LongOperationState::Cancelled("Export cancelled".into()),
        )));
        assert!(!state.is_export_busy());
    }
}

#[test]
fn cancellation_stays_busy_and_blocks_new_exports_until_cleanup_finishes() {
    let (_dir, mut state) = app_with_games();
    let game = all_games(&state).remove(0);
    open_and_select_all(&mut state);
    let run = state.begin_export_run();
    drop(state.update(Message::ExportProgress(
        run.id,
        LongOperationState::Progress("Writing game".into()),
    )));
    drop(state.update(Message::CancelExport));
    drop(state.update(Message::CancelExport));
    drop(state.update(Message::PickExportDest(game)));
    drop(state.update(Message::PickBatchExportDestination));
    drop(state.update(Message::ExportProgress(
        run.id,
        LongOperationState::Progress("Queued progress from a worker".into()),
    )));
    drop(state.update(Message::ExportProgress(run.id, LongOperationState::Idle)));
    drop(state.update(Message::SetLongOperationState(
        LongOperationKind::Export,
        LongOperationState::Finished("Unscoped old result".into()),
    )));
    assert!(state.export_picker.is_none());
    assert!(state.is_export_busy());
    assert_eq!(export_progress(&state), "Cancelling export...");
    assert!(!state.notifications.has_notifications());

    drop(state.update(Message::ExportProgress(
        run.id,
        LongOperationState::Cancelled("Export cancelled; completed games kept".into()),
    )));
    assert!(!state.is_export_busy());
    assert!(!state.is_export_cancelling());
    assert_eq!(state.notifications.iter().count(), 1);
    assert_eq!(
        state.notifications.iter().next().unwrap().level(),
        NotificationLevel::Info
    );
    assert!(state.begin_export_picker().is_some());
}

#[test]
fn closing_modal_or_changing_drive_preserves_the_active_export_token() {
    let (_dir, mut state) = app_with_games();
    open_and_select_all(&mut state);
    let run = state.begin_export_run();
    drop(state.update(Message::CloseModal));
    assert_eq!(state.active_export.as_ref().unwrap().id, run.id);
    let new_drive = tempfile::tempdir().expect("second drive");
    drop(state.update(Message::MountPointPicked(new_drive.path().to_path_buf())));
    assert_eq!(state.active_export.as_ref().unwrap().id, run.id);
    assert!(state.is_export_busy());
    drop(state.update(Message::CancelExport));
    assert!(run.cancellation.is_cancelled());
}

#[test]
fn stale_export_events_cannot_finish_or_modify_a_new_run() {
    let (_dir, mut state) = app_with_games();
    let old_run = state.begin_export_run();
    let terminal = LongOperationState::Cancelled("Old export cancelled".into());
    drop(state.update(Message::ExportProgress(old_run.id, terminal.clone())));
    drop(state.update(Message::ExportProgress(old_run.id, terminal.clone())));
    drop(state.update(Message::CancelExport));
    assert_eq!(state.notifications.iter().count(), 1);

    let new_run = state.begin_export_run();
    assert_ne!(old_run.id, new_run.id);
    drop(state.update(Message::ExportProgress(
        new_run.id,
        LongOperationState::Progress("New export".into()),
    )));
    for event in [
        terminal,
        LongOperationState::Progress("Old progress".into()),
        LongOperationState::Finished("Old completion".into()),
        LongOperationState::Errored("Old error".into()),
    ] {
        drop(state.update(Message::ExportProgress(old_run.id, event)));
    }
    assert_eq!(export_progress(&state), "New export");
    assert_eq!(state.active_export.as_ref().unwrap().id, new_run.id);
    assert!(!new_run.cancellation.is_cancelled());
    assert_eq!(state.notifications.iter().count(), 1);
}

#[test]
fn actual_terminal_result_wins_when_cancellation_races_with_completion() {
    for (terminal, level) in [
        (
            LongOperationState::Finished("Already exported".into()),
            NotificationLevel::Success,
        ),
        (
            LongOperationState::Errored("Cleanup failed".into()),
            NotificationLevel::Error,
        ),
        (
            LongOperationState::Cancelled("Cancelled".into()),
            NotificationLevel::Info,
        ),
    ] {
        let (_dir, mut state) = app_with_games();
        let run = state.begin_export_run();
        drop(state.update(Message::CancelExport));
        drop(state.update(Message::ExportProgress(run.id, terminal.clone())));
        drop(state.update(Message::ExportProgress(run.id, terminal)));
        assert!(!state.is_export_busy());
        assert_eq!(state.notifications.iter().count(), 1);
        assert_eq!(state.notifications.iter().next().unwrap().level(), level);
    }
}
