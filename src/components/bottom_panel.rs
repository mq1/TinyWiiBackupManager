// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::app::App;
use const_format::formatcp;
use eframe::egui;
use egui_suspense::EguiSuspense;
use semver::Version;
use std::sync::OnceLock;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const REPO: &str = env!("CARGO_PKG_REPOSITORY");
const VERSION_URL: &str = formatcp!("{REPO}/releases/latest/download/version.txt");

/// Information about an available update
#[derive(Debug, Clone)]
pub struct UpdateInfo {
    pub version: String,
    pub url: String,
}

/// Check for a newer version on GitHub using ehttp
fn check_for_new_version(cb: impl FnOnce(Result<Option<UpdateInfo>, String>) + Send + 'static) {
    let request = ehttp::Request::get(VERSION_URL);
    ehttp::fetch(request, move |result: Result<ehttp::Response, String>| {
        let response = match result {
            Ok(response) => response,
            Err(e) => {
                cb(Err(e)); // Pass the error to the callback
                return;
            }
        };

        let latest_version = match String::from_utf8(response.bytes) {
            Ok(version) => version,
            Err(e) => {
                cb(Err(e.to_string())); // Pass the error to the callback
                return;
            }
        };

        let latest = match Version::parse(&latest_version) {
            Ok(version) => version,
            Err(e) => {
                cb(Err(e.to_string())); // Pass the error to the callback
                return;
            }
        };

        let current = match Version::parse(VERSION) {
            Ok(version) => version,
            Err(e) => {
                cb(Err(e.to_string())); // Pass the error to the callback
                return;
            }
        };

        let update_info = (latest > current).then_some(UpdateInfo {
            version: format!("v{latest}"),
            url: format!("{REPO}/releases/tag/v{latest}"),
        });

        cb(Ok(update_info)); // Pass the result to the callback
    });
}

// Static storage for the version check result
static VERSION_CHECK_RESULT: OnceLock<Option<UpdateInfo>> = OnceLock::new();

/// Renders the bottom panel
pub fn ui_bottom_panel(ctx: &egui::Context, app: &mut App) {
    egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
        ui.horizontal(|ui| {
            // Check if we have already performed the version check
            match VERSION_CHECK_RESULT.get() {
                Some(result) => {
                    // We have the result, display it if there's an update
                    if let Some(update_info) = result {
                        let update_text = format!("⚠ Update available: {}", update_info.version);
                        ui.hyperlink_to(update_text, &update_info.url)
                            .on_hover_text("Update to the latest version");
                    }
                }
                None => {
                    // We haven't performed the check yet, create a suspense component to do it
                    let mut suspense = EguiSuspense::single_try(|cb| {
                        check_for_new_version(move |result| {
                            // Store the result for future frames
                            let _ = VERSION_CHECK_RESULT.set(result.clone().unwrap_or(None));
                            // Call the original callback
                            cb(result);
                        });
                    });
                    
                    suspense.ui(ui, |ui, data, _state| {
                        match data {
                            Some(update_info) => {
                                let update_text = format!("⚠ Update available: {}", update_info.version);
                                ui.hyperlink_to(update_text, &update_info.url)
                                    .on_hover_text("Update to the latest version");
                            }
                            None => {
                                // No update available or error occurred, show nothing
                            }
                        }
                    });
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.checkbox(&mut app.remove_sources, "💣 Remove sources")
                    .on_hover_text("⚠ DANGER ⚠\n\nThis will delete the input files!");
            });
        });
    });
}
