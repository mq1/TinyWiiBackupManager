use anyhow::Result;
use std::sync::mpsc::Receiver;
use std::task::Waker;

use crate::jobs::{Job, JobContext, JobResult, start_job, update_status};
use crate::update_check;

pub struct CheckUpdateConfig {}

pub struct CheckUpdateResult {
    pub update_available: bool,
    pub version: Option<String>,
    pub url: Option<String>,
}

fn run_check_update(
    context: &JobContext,
    cancel: Receiver<()>,
    _config: CheckUpdateConfig,
) -> Result<Box<CheckUpdateResult>> {
    update_status(context, "Checking for updates".to_string(), 0, 1, &cancel)?;

    match update_check::check_for_new_version()? {
        Some(update_info) => {
            update_status(context, "Update available".to_string(), 1, 1, &cancel)?;
            Ok(Box::new(CheckUpdateResult {
                update_available: true,
                version: Some(update_info.version),
                url: Some(update_info.url),
            }))
        }
        None => {
            update_status(context, "No updates available".to_string(), 1, 1, &cancel)?;
            Ok(Box::new(CheckUpdateResult {
                update_available: false,
                version: None,
                url: None,
            }))
        }
    }
}

pub fn start_check_update(waker: Waker, config: CheckUpdateConfig) -> super::JobState {
    start_job(
        waker,
        "Check for updates",
        Job::CheckUpdate,
        move |context, cancel| {
            run_check_update(&context, cancel, config).map(JobResult::CheckUpdate)
        },
    )
}
