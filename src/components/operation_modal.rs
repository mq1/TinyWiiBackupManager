use crate::app::App;
use crate::jobs::Job;
use eframe::egui;
use size::Size;

pub fn ui_operation_modal(ctx: &egui::Context, app: &mut App) {
    // Check if conversion or verification job is running
    let convert_job = app.jobs.get_job_by_kind(Job::Convert);
    let verify_job = app.jobs.get_job_by_kind(Job::Verify);

    let job_state = convert_job.or(verify_job);

    if let Some(job) = job_state {
        // Only show modal if job is actually running
        if job.handle.is_none() || job.handle.as_ref().unwrap().is_finished() {
            return;
        }

        let Ok(status) = job.context.status.read() else {
            return;
        };

        let modal_id = match job.kind {
            Job::Convert => "conversion_modal",
            Job::Verify => "verification_modal",
            _ => return,
        };

        egui::Modal::new(egui::Id::new(modal_id)).show(ctx, |ui| {
            ui.set_min_width(400.0);
            ui.vertical_centered(|ui| {
                // Header
                let heading = match job.kind {
                    Job::Convert => "📦 Converting Files",
                    Job::Verify => "🔍 Verifying Disc Integrity",
                    _ => "",
                };
                ui.heading(heading);
                ui.separator();
                ui.add_space(10.0);

                // Show overall progress if processing multiple items
                if let Some([current_idx, total_items]) = status.progress_items {
                    if total_items > 1 {
                        let item_label = match job.kind {
                            Job::Convert => "File",
                            Job::Verify => "Game",
                            _ => "Item",
                        };
                        ui.label(format!(
                            "{} {} of {}",
                            item_label,
                            current_idx + 1,
                            total_items
                        ));
                        ui.add_space(5.0);
                    }
                }

                // Show current item name
                if let Some(item_name) = &status.current_item_name {
                    ui.label(egui::RichText::new(item_name).strong());
                    ui.add_space(5.0);
                }

                ui.spinner();
                ui.add_space(10.0);

                if let Some([current, total]) = status.progress_bytes {
                    if total > 0 {
                        // Use the pre-calculated progress from the job
                        ui.add(egui::ProgressBar::new(status.progress_percent).show_percentage());
                        ui.add_space(5.0);

                        ui.label(format!(
                            "Progress: {} / {}",
                            Size::from_bytes(current),
                            Size::from_bytes(total)
                        ));
                    } else {
                        let initializing_msg = match job.kind {
                            Job::Convert => "Initializing conversion...",
                            Job::Verify => "Initializing verification...",
                            _ => "Initializing...",
                        };
                        ui.label(initializing_msg);
                    }
                } else {
                    // No byte progress yet
                    ui.label(&status.status);
                }

                ui.add_space(10.0);

                // Cancel button
                if ui.button("Cancel").clicked() {
                    if let Err(e) = job.cancel.send(()) {
                        log::error!("Failed to cancel job: {e:?}");
                    }
                }
            });
        });
    }
}
