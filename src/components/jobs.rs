use crate::jobs::{JobQueue, JobStatus};
use eframe::egui::{self, ProgressBar, RichText, Widget};
use size::Size;
use std::cmp::Ordering;

pub fn jobs_ui(ui: &mut egui::Ui, jobs: &mut JobQueue) {
    if ui.button("Clear").clicked() {
        jobs.clear_errored();
    }

    let mut remove_job: Option<usize> = None;
    let mut any_jobs = false;
    for job in jobs.iter_mut() {
        let Ok(status) = job.context.status.read() else {
            continue;
        };
        any_jobs = true;
        ui.separator();
        ui.horizontal(|ui| {
            ui.label(&status.title);
            if ui.small_button("✖").clicked() {
                if job.handle.is_some() {
                    if let Err(e) = job.cancel.send(()) {
                        log::error!("Failed to cancel job: {e:?}");
                    }
                } else {
                    remove_job = Some(job.id);
                }
            }
        });
        let mut bar = ProgressBar::new(status.progress_percent);
        if let Some(bytes) = status.progress_bytes {
            let mut text = format!(
                "{} / {}",
                Size::from_bytes(bytes[0]),
                Size::from_bytes(bytes[1])
            );
            if let Some(items) = status.progress_items
                && items[1] > 1
            {
                text.push_str(&format!(" ({} / {})", items[0], items[1]));
            }
            bar = bar.text(text);
        } else if let Some(items) = status.progress_items {
            bar = bar.text(format!("{} / {}", items[0], items[1]));
        }
        bar.ui(ui);
        const STATUS_LENGTH: usize = 60;
        if let Some(err) = &status.error {
            let err_string = format!("{err:#}");
            let delete_color = ui.visuals().error_fg_color;
            ui.colored_label(
                delete_color,
                if err_string.len() > STATUS_LENGTH - 10 {
                    format!("Error: {}…", &err_string[0..STATUS_LENGTH - 10])
                } else {
                    format!("Error: {:width$}", err_string, width = STATUS_LENGTH - 7)
                },
            )
            .on_hover_text_at_pointer(RichText::new(&err_string).color(delete_color))
            .context_menu(|ui| {
                if ui.button("Copy full message").clicked() {
                    ui.ctx().copy_text(err_string);
                }
            });
        } else {
            ui.label(if status.status.len() > STATUS_LENGTH - 3 {
                format!("{}…", &status.status[0..STATUS_LENGTH - 3])
            } else {
                format!("{:width$}", &status.status, width = STATUS_LENGTH)
            })
            .on_hover_text_at_pointer(&status.status)
            .context_menu(|ui| {
                if ui.button("Copy full message").clicked() {
                    ui.ctx().copy_text(status.status.clone());
                }
            });
        }
    }
    if !any_jobs {
        ui.label("No jobs");
    }

    if let Some(idx) = remove_job {
        jobs.remove(idx);
    }
}

struct JobStatusDisplay {
    title: String,
    progress_items: Option<[u32; 2]>,
    error: bool,
}

impl From<&JobStatus> for JobStatusDisplay {
    fn from(status: &JobStatus) -> Self {
        Self {
            title: status.title.clone(),
            progress_items: status.progress_items,
            error: status.error.is_some(),
        }
    }
}

pub fn jobs_menu_ui(ui: &mut egui::Ui, jobs: &mut JobQueue) -> bool {
    let mut statuses = Vec::new();
    for job in jobs.iter_mut() {
        let Ok(status) = job.context.status.read() else {
            continue;
        };
        statuses.push(JobStatusDisplay::from(&*status));
    }
    let running_jobs = statuses.iter().filter(|s| !s.error).count();
    let error_jobs = statuses.iter().filter(|s| s.error).count();

    let mut clicked = false;
    let spinner = egui::Spinner::new().size(ui.text_style_height(&egui::TextStyle::Body) * 0.9);
    match running_jobs.cmp(&1) {
        Ordering::Equal => {
            spinner.ui(ui);
            let running_job = statuses.iter().find(|s| !s.error).unwrap();
            let text = if let Some(items) = running_job.progress_items {
                format!("{} ({}/{})", running_job.title, items[0], items[1])
            } else {
                running_job.title.clone()
            };
            clicked |= ui.link(RichText::new(text)).clicked();
        }
        Ordering::Greater => {
            spinner.ui(ui);
            clicked |= ui.link(format!("{running_jobs} running")).clicked();
        }
        _ => (),
    }
    match error_jobs.cmp(&1) {
        Ordering::Equal => {
            let error_job = statuses.iter().find(|s| s.error).unwrap();
            let error_color = ui.visuals().error_fg_color;
            clicked |= ui
                .link(RichText::new(format!("{} error", error_job.title)).color(error_color))
                .clicked();
        }
        Ordering::Greater => {
            let error_color = ui.visuals().error_fg_color;
            clicked |= ui
                .link(RichText::new(format!("{error_jobs} errors")).color(error_color))
                .clicked();
        }
        _ => (),
    }
    if running_jobs == 0 && error_jobs == 0 {
        clicked |= ui.link("None").clicked();
    }
    clicked
}

pub fn jobs_window(ctx: &egui::Context, show: &mut bool, jobs: &mut JobQueue) {
    egui::Window::new("Jobs").open(show).show(ctx, |ui| {
        jobs_ui(ui, jobs);
    });
}
