use crate::app::App;
use crate::components;
use crate::jobs::{Job, JobResult};
use crate::messages::handle_messages;
use eframe::{Storage, egui};

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Process job results
        self.jobs.collect_results();
        let results = std::mem::take(&mut self.jobs.results);
        for (result, status) in results {
            match result {
                JobResult::DownloadCovers(res) => self.handle_download_covers_result(status, *res),
                JobResult::DownloadDatabase(res) => {
                    self.handle_download_database_result(status, *res)
                }
                JobResult::Convert(res) => self.handle_convert_result(status, *res),
                JobResult::Verify(res) => self.handle_verify_result(status, *res),
                // Put back unhandled results
                _ => self.jobs.results.push((result, status)),
            }
        }

        handle_messages(self, ctx);

        // Set cursor icon based on active jobs
        if self.jobs.is_running(Job::Convert) || self.jobs.is_running(Job::Verify) {
            ctx.set_cursor_icon(egui::CursorIcon::Wait);
        } else {
            ctx.set_cursor_icon(egui::CursorIcon::Default);
        }

        components::top_panel::ui_top_panel(ctx, self);
        components::bottom_panel::ui_bottom_panel(ctx, self);

        egui::CentralPanel::default().show(ctx, |ui| {
            components::game_grid::ui_game_grid(ui, self);

            // Show operation modal if convert or verify job is running
            components::operation_modal::ui_operation_modal(ctx, self);
        });

        // Render info windows for opened games and remove closed ones
        self.open_info_windows.retain(|&index| {
            self.games.get_mut(index).is_some_and(|game| {
                let mut is_open = true;
                components::game_info::ui_game_info_window(
                    ctx,
                    game,
                    &mut is_open,
                    self.inbox.sender(),
                );
                is_open
            })
        });

        // Render jobs window if open
        components::jobs::jobs_window(ctx, &mut self.show_jobs_window, &mut self.jobs);

        self.top_left_toasts.show(ctx);
        self.bottom_left_toasts.show(ctx);
        self.bottom_right_toasts.show(ctx);
    }

    fn save(&mut self, storage: &mut dyn Storage) {
        if let Some(base_dir) = &self.base_dir {
            eframe::set_value(storage, "base_dir", base_dir);
        }
    }

    fn persist_egui_memory(&self) -> bool {
        false
    }
}
