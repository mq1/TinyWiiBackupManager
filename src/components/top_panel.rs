// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::app::App;
use crate::components::fake_link::fake_link;
use crate::cover_manager::CoverType;
use crate::game::VerificationStatus;
use crate::jobs::{Job, download_covers, download_database, egui_waker};
use crate::messages::BackgroundMessage;
use anyhow::anyhow;
use eframe::egui;
use size::Size;

/// Renders the top menu bar.
pub fn ui_top_panel(ctx: &egui::Context, app: &mut App) {
    let sender = app.inbox.sender();

    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("📄 File", |ui| {
                // remove hint toast
                app.top_left_toasts.dismiss_all_toasts();

                // Re-pick base directory button
                if ui.button("📁 Pick base Drive/Directory").clicked()
                    && let Err(e) = app.choose_base_dir()
                {
                    let _ = sender.send(BackgroundMessage::Error(e));
                }

                // dot_clean button
                if cfg!(target_os = "macos")
                    && let Some(base_dir) = &app.base_dir
                    && ui
                        .button("👻 Clean MacOS ._ files")
                        .on_hover_text(format!("Run dot_clean in {base_dir}"))
                        .clicked()
                    && let Err(e) = base_dir.run_dot_clean()
                {
                    let _ = sender.send(BackgroundMessage::Error(e));
                }
            });

            if app.base_dir.is_some() {
                let add_games_button = ui
                    .button("➕ Add Game(s)")
                    .on_hover_text("Add one or more games to the wbfs directory");

                if add_games_button.clicked() {
                    app.add_isos();
                }

                // Verify All button - disable if all games are verified
                let has_unverified = app.games.iter().any(|g| {
                    !matches!(
                        g.get_verification_status(),
                        VerificationStatus::FullyVerified(_, _)
                    )
                });

                let verify_all_button = ui
                    .add_enabled(has_unverified, egui::Button::new("🔍 Verify All"))
                    .on_hover_text(if has_unverified {
                        "Verify integrity of all games"
                    } else {
                        "All games are already verified"
                    });

                if verify_all_button.clicked() {
                    app.start_verify_all();
                }

                // GameTDB menu
                ui.label("•");
                ui.menu_button("🎮 GameTDB", |ui| {
                    // Download database option
                    if ui
                        .button("📥 Update GameTDB Database")
                        .on_hover_text("Download the latest wiitdb.xml database from GameTDB")
                        .clicked()
                        && let Some(cover_manager) = &app.cover_manager
                    {
                        let config = download_database::DownloadDatabaseConfig {
                            base_dir: cover_manager.base_dir().clone(),
                        };
                        app.jobs.push_once(Job::DownloadDatabase, || {
                            download_database::start_download_database(
                                egui_waker::egui_waker(ctx),
                                config,
                            )
                        });
                        app.bottom_right_toasts
                            .info("Updating GameTDB database...");
                    }

                    ui.separator();

                    // Download all covers options
                    download_covers_ui(ui, ctx, app, CoverType::Cover3D);
                    download_covers_ui(ui, ctx, app, CoverType::Cover2D);
                    download_covers_ui(ui, ctx, app, CoverType::CoverFull);
                    download_covers_ui(ui, ctx, app, CoverType::Disc);
                });
            }

            // Tests (only debug builds)
            if cfg!(debug_assertions) {
                ui.label("•");
                ui.menu_button("🛠 Tests", |ui| {
                    if ui.button("❌ Test Error").clicked() {
                        let _ = sender.send(BackgroundMessage::Error(
                            anyhow!("Test error")
                                .context("Doing something")
                                .context("In ui_top_panel"),
                        ));
                    }

                    if ui.button("❌ Test Error 2").clicked() {
                        rfd::MessageDialog::new()
                            .set_title("Test Error 2")
                            .set_level(rfd::MessageLevel::Error)
                            .show();
                    }
                });
            }

            // Display the total number of games on the right side of the menu bar
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.hyperlink_to("ℹ Wiki", "https://github.com/mq1/TinyWiiBackupManager/wiki")
                    .on_hover_text("Open the TinyWiiBackupManager wiki");

                ui.label("•");

                if let Some(base_dir) = &app.base_dir {
                    ui.label(format!("Size: {}", Size::from_bytes(app.base_dir_size)));

                    let base_dir_name = base_dir.name();
                    if fake_link(ui, &base_dir_name)
                        .on_hover_text(format!("Open the base directory ({base_dir_name})"))
                        .clicked()
                        && let Err(e) = base_dir.open()
                    {
                        let _ = sender.send(BackgroundMessage::Error(e));
                    }

                    ui.label("•");
                }

                ui.label(format!("{} games", app.games.len()));
            });
        });
    });
}

/// Helper function to handle downloading covers of a specific type
fn download_covers_ui(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    app: &mut App,
    cover_type: CoverType,
) {
    let cover_type_name = match cover_type {
        CoverType::Cover3D => "3D covers",
        CoverType::Cover2D => "2D covers",
        CoverType::CoverFull => "full covers",
        CoverType::Disc => "disc art",
    };
    if ui
        .button(format!("📥 Download all {cover_type_name}"))
        .on_hover_text(format!("Download {cover_type_name} for all games"))
        .clicked()
        && let Some(cover_manager) = &app.cover_manager
    {
        let game_ids: Vec<String> = app
            .games
            .iter()
            .map(|g| g.id.clone())
            .filter(|id| !cover_manager.has_cover(id, cover_type))
            .filter(|id| !cover_manager.is_failed(id, cover_type))
            .collect();

        if game_ids.is_empty() {
            app.bottom_right_toasts
                .info(format!("All {} already downloaded.", cover_type_name));
        } else {
            let num_covers = game_ids.len();
            let config = download_covers::DownloadCoversConfig {
                base_dir: cover_manager.base_dir().clone(),
                cover_type,
                game_ids,
            };
            app.jobs.push_once(Job::DownloadCovers, || {
                download_covers::start_download_covers(egui_waker::egui_waker(ctx), config)
            });
            app.bottom_right_toasts.info(format!(
                "Downloading {} for {} games...",
                cover_type_name, num_covers
            ));
        }
    }
}
