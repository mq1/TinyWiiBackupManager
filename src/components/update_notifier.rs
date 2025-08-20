// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use anyhow::{Context, Result, anyhow, bail};
use const_format::formatcp;
use eframe::egui;
use egui_suspense::EguiSuspense;
use semver::Version;
use std::sync::{Mutex, OnceLock};

// --- Constants ---
const VERSION: &str = env!("CARGO_PKG_VERSION");
const REPO: &str = env!("CARGO_PKG_REPOSITORY");
const VERSION_URL: &str = formatcp!("{REPO}/releases/latest/download/version.txt");

// --- Data Structures ---

/// Holds information about a newer, available version of the application.
#[derive(Debug, Clone)]
pub struct UpdateInfo {
    pub version: String,
    pub url: String,
}

// --- Static State ---

/// A thread-safe, lazily-initialized component for checking for updates.
///
/// # Design
/// - `OnceLock` ensures the `EguiSuspense` component is created only once.
/// - `Mutex` provides safe interior mutability. This is required because the `.ui()` method
///   needs a mutable reference (`&mut self`), but `get_or_init` can only provide a shared
///   reference (`&self`). The mutex allows us to safely acquire mutable access.
static UPDATE_CHECKER: OnceLock<Mutex<EguiSuspense<Option<UpdateInfo>, anyhow::Error>>> =
    OnceLock::new();

// --- UI Rendering ---

/// Renders the update notifier UI component.
pub fn ui(ui: &mut egui::Ui) {
    // Get or initialize the update checker. This is cheap after the first call.
    let suspense_mutex =
        UPDATE_CHECKER.get_or_init(|| Mutex::new(EguiSuspense::single_try(check_for_new_version)));

    // Lock the mutex to get mutable access. `unwrap` is acceptable here as a poisoned
    // mutex is a non-recoverable state for this UI component.
    let mut suspense = suspense_mutex.lock().unwrap();

    // Render the suspense UI. It will only draw its contents when the async task succeeds.
    suspense.ui(ui, |ui, data, _state| {
        if let Some(update_info) = data {
            let update_text = format!("⚠ Update available: {}", update_info.version);
            ui.hyperlink_to(update_text, &update_info.url)
                .on_hover_text("Click to open the latest release page");
        }
    });
}

// --- Asynchronous Logic ---

/// Asynchronously checks for a newer version on GitHub.
///
/// This function is designed to be called once by `EguiSuspense`. It uses `anyhow`
/// for ergonomic error handling. On failure, errors are logged but not shown in the UI.
fn check_for_new_version(cb: impl FnOnce(Result<Option<UpdateInfo>>) + Send + 'static) {
    let request = ehttp::Request::get(VERSION_URL);

    ehttp::fetch(request, move |response| {
        // Use an immediately-invoked closure to leverage the `?` operator.
        let result = (|| {
            let response = response.map_err(|e| anyhow!(e))?;

            if !response.ok {
                bail!(
                    "HTTP request failed: {} {}",
                    response.status,
                    response.status_text
                );
            }

            let latest_version_str = String::from_utf8(response.bytes)
                .context("Failed to decode response body as UTF-8")?;

            let current_version = Version::parse(VERSION)
                .context(format!("Failed to parse current version: '{VERSION}'"))?;
            let latest_version = Version::parse(latest_version_str.trim()).context(format!(
                "Failed to parse latest version from server: '{latest_version_str}'"
            ))?;

            // If the latest version is greater, create the update info. Otherwise, return None.
            let update_info = (latest_version > current_version).then_some(UpdateInfo {
                version: format!("v{latest_version}"),
                url: format!("{REPO}/releases/tag/v{latest_version}"),
            });

            Ok(update_info)
        })();

        // If any step in the closure failed, log the detailed error chain.
        if let Err(e) = &result {
            // Use `{:?}` with anyhow to get a full, multi-line error report with causes.
            log::error!("Failed to check for updates: {:?}", e);
        }

        cb(result);
    });
}
