// SPDX-FileCopyrightText: 2026 TinyWiiBackupManager contributors
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::{SortBy, message::ConfigMessage},
    games::{
        calc_sha1::{BatchHashEvent, HashOutcome, HashResult, HashSummary},
        game::Game,
        games_state::GamesState,
    },
    long_operation::{LongOperationKind, LongOperationState},
    message::Message,
    state::AppState,
    ui::modals::{Modal, check_games::HashSelection},
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

    // Boot and refresh tasks stay unpolled: no dialogs, network requests, or
    // hashing threads are started by these state regression tests.
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

fn selection(state: &AppState) -> &HashSelection {
    state.hash_batch.as_ref().expect("expected hash selection")
}

fn open_and_select_all(state: &mut AppState) {
    drop(state.update(Message::OpenHashGames));
    drop(state.update(Message::SelectAllHashGames(true)));
}

fn simulate_running_batch(state: &mut AppState, run_id: u64) {
    open_and_select_all(state);
    state.hash_batch.as_mut().unwrap().start(run_id);
    state.long_operations.set(
        LongOperationKind::Hash,
        LongOperationState::Progress("Hashing selected games".into()),
    );
}

fn result(game: Game, outcome: HashOutcome) -> HashResult {
    HashResult {
        game,
        disc: Some(1),
        sha1: Some("0123456789abcdef0123456789abcdef01234567".into()),
        outcome,
    }
}

#[test]
fn hash_selection_includes_filtered_games_and_distinguishes_duplicate_ids() {
    let (_dir, mut state) = app_with_games();
    let expected = all_games(&state);
    drop(state.update(Message::SearchGames("no matching game".into())));
    drop(state.update(Message::ToggleShowWii(false)));
    drop(state.update(Message::ToggleShowNgc(false)));
    let GamesState::Loaded(games) = &state.games else {
        panic!("expected loaded games");
    };
    assert_eq!(games.iter_by(SortBy::NameAscending).count(), 0);

    open_and_select_all(&mut state);
    assert_eq!(selection(&state).selected_games(), expected);
    drop(state.update(Message::SelectAllHashGames(false)));
    assert_eq!(selection(&state).selected_count(), 0);

    let duplicate_indices: Vec<_> = expected
        .iter()
        .enumerate()
        .filter_map(|(index, game)| game.is_wii().then_some(index))
        .collect();
    let (first, second) = (duplicate_indices[0], duplicate_indices[1]);
    assert_eq!(expected[first].id(), expected[second].id());
    assert_ne!(expected[first].path(), expected[second].path());

    drop(state.update(Message::ToggleHashGame(first, true)));
    assert_eq!(
        selection(&state).selected_games(),
        vec![expected[first].clone()]
    );
    drop(state.update(Message::ToggleHashGame(second, true)));
    drop(state.update(Message::ToggleHashGame(first, false)));
    assert_eq!(
        selection(&state).selected_games(),
        vec![expected[second].clone()]
    );
}

#[test]
fn empty_hash_selection_cannot_start_a_worker() {
    let (_dir, mut state) = app_with_games();
    drop(state.update(Message::OpenHashGames));
    let task = state.update(Message::CheckSelectedGames);
    assert_eq!(task.units(), 0);
    assert!(!selection(&state).running);
    assert_eq!(selection(&state).run_id, 0);
    assert!(state.long_operations.is_idle(LongOperationKind::Hash));
}

#[test]
fn refreshing_games_does_not_replace_the_hash_selection_snapshot() {
    let (_dir, mut state) = app_with_games();
    open_and_select_all(&mut state);
    drop(state.update(Message::ToggleHashGame(0, false)));
    let expected = selection(&state).selected_games();
    drop(state.update(Message::GotGames(GamesState::NotLoaded)));
    assert_eq!(selection(&state).selected_games(), expected);
    assert_eq!(selection(&state).selected_count(), 2);
}

#[test]
fn active_hash_blocks_single_and_batch_starts_and_selection_edits() {
    let (_dir, mut state) = app_with_games();
    simulate_running_batch(&mut state, 11);
    let games = selection(&state).selected_games();

    assert_eq!(
        state
            .update(Message::CalcGameSha1(games[0].clone()))
            .units(),
        0
    );
    assert_eq!(state.update(Message::CheckSelectedGames).units(), 0);
    drop(state.update(Message::ToggleHashGame(0, false)));
    drop(state.update(Message::SelectAllHashGames(false)));
    drop(state.update(Message::NewHashSelection));

    assert_eq!(selection(&state).selected_games(), games);
    assert_eq!(selection(&state).run_id, 11);
    assert!(selection(&state).running);
    assert!(!state.long_operations.is_idle(LongOperationKind::Hash));
}

#[test]
fn an_existing_single_hash_also_blocks_starting_a_batch() {
    let (_dir, mut state) = app_with_games();
    open_and_select_all(&mut state);
    state.long_operations.set(
        LongOperationKind::Hash,
        LongOperationState::Progress("Hashing a single game".into()),
    );

    assert_eq!(state.update(Message::CheckSelectedGames).units(), 0);
    assert!(!selection(&state).running);
    assert_eq!(selection(&state).run_id, 0);
    assert!(!state.long_operations.is_idle(LongOperationKind::Hash));
}

#[test]
fn closing_and_reopening_keeps_the_active_report_and_results() {
    let (dir, mut state) = app_with_games();
    simulate_running_batch(&mut state, 12);
    let game = selection(&state).selected_games().remove(0);
    drop(state.update(Message::CloseModal));
    assert!(state.current_modal.is_none());
    drop(state.update(Message::HashBatchUpdated(
        12,
        BatchHashEvent::Result(result(game.clone(), HashOutcome::Unknown)),
    )));
    assert_eq!(selection(&state).results.len(), 1);
    assert!(!state.notifications.has_notifications());

    drop(state.update(Message::OpenHashGames));
    assert!(matches!(state.current_modal, Some(Modal::CheckGames)));
    assert_eq!(selection(&state).source, dir.path());
    assert_eq!(selection(&state).run_id, 12);
    assert!(selection(&state).running);
    assert_eq!(selection(&state).results[0].game, game);
    assert!(!state.long_operations.is_idle(LongOperationKind::Hash));
}

#[test]
fn disc_results_keep_hash_busy_and_only_completion_notifies() {
    let (_dir, mut state) = app_with_games();
    simulate_running_batch(&mut state, 13);
    let games = selection(&state).selected_games();
    drop(state.update(Message::HashBatchUpdated(
        13,
        BatchHashEvent::Progress("Hashing 2/3: 40%".into()),
    )));
    assert_eq!(selection(&state).progress, "Hashing 2/3: 40%");

    for (game, outcome) in games.iter().cloned().zip([
        HashOutcome::Known,
        HashOutcome::Unknown,
        HashOutcome::Error("Disc not found".into()),
    ]) {
        drop(state.update(Message::HashBatchUpdated(
            13,
            BatchHashEvent::Result(result(game, outcome)),
        )));
        assert!(selection(&state).running);
        assert!(!state.long_operations.is_idle(LongOperationKind::Hash));
        assert!(!state.notifications.has_notifications());
    }
    assert_eq!(selection(&state).results.len(), 3);

    let summary = HashSummary {
        games: 3,
        known: 1,
        unknown: 1,
        errors: 1,
    };
    drop(state.update(Message::HashBatchUpdated(
        13,
        BatchHashEvent::Finished(summary.clone()),
    )));
    assert!(!selection(&state).running);
    assert_eq!(selection(&state).summary.as_ref(), Some(&summary));
    assert_eq!(selection(&state).results.len(), 3);
    assert!(state.long_operations.is_idle(LongOperationKind::Hash));
    assert_eq!(state.notifications.iter().count(), 1);

    drop(state.update(Message::CloseModal));
    drop(state.update(Message::OpenHashGames));
    assert_eq!(selection(&state).summary.as_ref(), Some(&summary));
    assert_eq!(selection(&state).results.len(), 3);
}

#[test]
fn stale_and_post_completion_hash_events_do_not_change_the_report() {
    let (_dir, mut state) = app_with_games();
    simulate_running_batch(&mut state, 14);
    let game = selection(&state).selected_games().remove(0);
    let original_progress = selection(&state).progress.clone();
    for event in [
        BatchHashEvent::Progress("Stale progress".into()),
        BatchHashEvent::Result(result(game.clone(), HashOutcome::Known)),
        BatchHashEvent::Finished(HashSummary::default()),
    ] {
        drop(state.update(Message::HashBatchUpdated(13, event)));
    }
    assert!(selection(&state).running);
    assert!(selection(&state).results.is_empty());
    assert!(selection(&state).summary.is_none());
    assert_eq!(selection(&state).progress, original_progress);
    assert!(!state.long_operations.is_idle(LongOperationKind::Hash));
    assert!(!state.notifications.has_notifications());

    let summary = HashSummary {
        games: 3,
        known: 3,
        unknown: 0,
        errors: 0,
    };
    drop(state.update(Message::HashBatchUpdated(
        14,
        BatchHashEvent::Finished(summary.clone()),
    )));
    let completed_progress = selection(&state).progress.clone();
    for event in [
        BatchHashEvent::Progress("Late progress".into()),
        BatchHashEvent::Result(result(game, HashOutcome::Unknown)),
        BatchHashEvent::Finished(HashSummary::default()),
    ] {
        drop(state.update(Message::HashBatchUpdated(14, event)));
    }
    assert!(!selection(&state).running);
    assert!(selection(&state).results.is_empty());
    assert_eq!(selection(&state).summary.as_ref(), Some(&summary));
    assert_eq!(selection(&state).progress, completed_progress);
    assert!(state.long_operations.is_idle(LongOperationKind::Hash));
    assert_eq!(state.notifications.iter().count(), 1);
}

#[test]
fn changing_drive_discards_only_a_never_started_hash_selection() {
    let (_dir, mut state) = app_with_games();
    open_and_select_all(&mut state);
    let new_drive = tempfile::tempdir().unwrap();
    drop(state.update(Message::MountPointPicked(new_drive.path().to_path_buf())));
    assert!(state.current_modal.is_none());
    assert!(state.hash_batch.is_none());
    assert!(state.long_operations.is_idle(LongOperationKind::Hash));
}

#[test]
fn changing_drive_preserves_active_report_and_new_selection_uses_current_drive() {
    let (dir, mut state) = app_with_games();
    simulate_running_batch(&mut state, 15);
    let original_games = selection(&state).selected_games();
    let new_drive = tempfile::tempdir().unwrap();
    let new_path = new_drive.path().join("games/New game [GZZE01]");
    std::fs::create_dir_all(new_path).unwrap();
    drop(state.update(Message::Config(ConfigMessage::SetMountPoint(
        new_drive.path().to_path_buf(),
    ))));
    assert!(state.current_modal.is_none());
    assert_eq!(selection(&state).source, dir.path());
    assert_eq!(selection(&state).selected_games(), original_games);
    assert!(selection(&state).running);
    assert!(!state.long_operations.is_idle(LongOperationKind::Hash));

    state.games = smol::block_on(GamesState::load(new_drive.path().to_path_buf()));
    let new_games = all_games(&state);
    drop(state.update(Message::OpenHashGames));
    assert!(matches!(state.current_modal, Some(Modal::CheckGames)));
    assert_eq!(selection(&state).source, dir.path());
    drop(state.update(Message::NewHashSelection));
    assert_eq!(selection(&state).run_id, 15);

    let summary = HashSummary {
        games: 3,
        known: 2,
        unknown: 1,
        errors: 0,
    };
    drop(state.update(Message::HashBatchUpdated(
        15,
        BatchHashEvent::Finished(summary.clone()),
    )));
    drop(state.update(Message::CloseModal));
    drop(state.update(Message::OpenHashGames));
    assert_eq!(selection(&state).source, dir.path());
    assert_eq!(selection(&state).summary.as_ref(), Some(&summary));

    drop(state.update(Message::NewHashSelection));
    assert_eq!(selection(&state).source, new_drive.path());
    assert_eq!(selection(&state).run_id, 0);
    assert!(!selection(&state).running);
    assert!(selection(&state).results.is_empty());
    assert!(selection(&state).summary.is_none());
    drop(state.update(Message::SelectAllHashGames(true)));
    assert_eq!(selection(&state).selected_games(), new_games);
}
