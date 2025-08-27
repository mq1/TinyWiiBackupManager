use std::{
    sync::{
        Arc, RwLock,
        atomic::{AtomicUsize, Ordering},
        mpsc::{Receiver, Sender, TryRecvError},
    },
    task::Waker,
    thread::JoinHandle,
};

use anyhow::Result;

pub mod download_covers;
pub mod download_database;
pub mod egui_waker;

use download_covers::DownloadCoversResult;
use download_database::DownloadDatabaseResult;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Job {
    DownloadCovers,
    DownloadDatabase,
}

pub static JOB_ID: AtomicUsize = AtomicUsize::new(0);

#[derive(Default)]
pub struct JobQueue {
    pub jobs: Vec<JobState>,
    pub results: Vec<JobResult>,
}

impl JobQueue {
    /// Adds a job to the queue.
    #[inline]
    pub fn push(&mut self, state: JobState) {
        self.jobs.push(state);
    }

    /// Adds a job to the queue if a job of the given kind is not already running.
    #[inline]
    pub fn push_once(&mut self, job: Job, func: impl FnOnce() -> JobState) {
        if !self.is_running(job) {
            self.push(func());
        }
    }

    /// Returns whether a job of the given kind is running.
    pub fn is_running(&self, kind: Job) -> bool {
        self.jobs
            .iter()
            .any(|j| j.kind == kind && j.handle.is_some())
    }

    /// Returns whether any job is running.
    pub fn any_running(&self) -> bool {
        self.jobs.iter().any(|job| {
            if let Some(handle) = &job.handle {
                return !handle.is_finished();
            }
            false
        })
    }

    /// Returns the number of active jobs.
    pub fn active_count(&self) -> usize {
        self.jobs
            .iter()
            .filter(|j| j.handle.as_ref().map_or(false, |h| !h.is_finished()))
            .count()
    }

    /// Iterates over all jobs mutably.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut JobState> + '_ {
        self.jobs.iter_mut()
    }

    /// Iterates over all finished jobs, returning the job state and the result.
    pub fn iter_finished(
        &mut self,
    ) -> impl Iterator<Item = (&mut JobState, std::thread::Result<JobResult>)> + '_ {
        self.jobs.iter_mut().filter_map(|job| {
            if let Some(handle) = &job.handle {
                if !handle.is_finished() {
                    return None;
                }
                let result = job.handle.take().unwrap().join();
                return Some((job, result));
            }
            None
        })
    }

    /// Clears all finished jobs.
    pub fn clear_finished(&mut self) {
        self.jobs.retain(|job| {
            !(job.handle.is_none() && job.context.status.read().unwrap().error.is_none())
        });
    }

    /// Clears all errored jobs.
    pub fn clear_errored(&mut self) {
        self.jobs
            .retain(|job| job.context.status.read().unwrap().error.is_none());
    }

    /// Removes a job from the queue given its ID.
    pub fn remove(&mut self, id: usize) {
        self.jobs.retain(|job| job.id != id);
    }

    /// Collects the results of all finished jobs and handles any errors.
    pub fn collect_results(&mut self) {
        let mut results = vec![];
        for (job, result) in self.iter_finished() {
            match result {
                Ok(result) => {
                    match result {
                        JobResult::None => {
                            // Job context contains the error
                        }
                        _ => results.push(result),
                    }
                }
                Err(err) => {
                    let err = if let Some(msg) = err.downcast_ref::<&'static str>() {
                        anyhow::Error::msg(*msg)
                    } else if let Some(msg) = err.downcast_ref::<String>() {
                        anyhow::Error::msg(msg.clone())
                    } else {
                        anyhow::Error::msg("Thread panicked")
                    };
                    let result = job.context.status.write();
                    if let Ok(mut guard) = result {
                        guard.error = Some(err);
                    } else {
                        drop(result);
                        job.context.status = Arc::new(RwLock::new(JobStatus {
                            title: "Error".to_string(),
                            progress_percent: 0.0,
                            progress_items: None,
                            status: String::new(),
                            error: Some(err),
                        }));
                    }
                }
            }
        }
        self.results.append(&mut results);
        self.clear_finished();
    }
}

#[derive(Clone)]
pub struct JobContext {
    pub status: Arc<RwLock<JobStatus>>,
    pub waker: Waker,
}

pub struct JobState {
    pub id: usize,
    pub kind: Job,
    pub handle: Option<JoinHandle<JobResult>>,
    pub context: JobContext,
    pub cancel: Sender<()>,
}

#[derive(Default)]
pub struct JobStatus {
    pub title: String,
    pub progress_percent: f32,
    pub progress_items: Option<[u32; 2]>,
    pub status: String,
    pub error: Option<anyhow::Error>,
}

pub enum JobResult {
    None,
    DownloadCovers(Box<DownloadCoversResult>),
    DownloadDatabase(Box<DownloadDatabaseResult>),
}

pub fn start_job(
    waker: Waker,
    title: &str,
    kind: Job,
    run: impl FnOnce(JobContext, Receiver<()>) -> Result<JobResult> + Send + 'static,
) -> JobState {
    let status = Arc::new(RwLock::new(JobStatus {
        title: title.to_string(),
        progress_percent: 0.0,
        progress_items: None,
        status: String::new(),
        error: None,
    }));
    let context = JobContext {
        status: status.clone(),
        waker: waker.clone(),
    };
    let context_inner = JobContext {
        status: status.clone(),
        waker,
    };
    let (tx, rx) = std::sync::mpsc::channel();
    let handle = std::thread::spawn(move || match run(context_inner, rx) {
        Ok(state) => state,
        Err(e) => {
            if let Ok(mut w) = status.write() {
                w.error = Some(e);
            }
            JobResult::None
        }
    });
    let id = JOB_ID.fetch_add(1, Ordering::Relaxed);
    JobState {
        id,
        kind,
        handle: Some(handle),
        context,
        cancel: tx,
    }
}

pub fn update_status(
    context: &JobContext,
    status: String,
    current: u32,
    total: u32,
    cancel: &Receiver<()>,
) -> Result<()> {
    let mut w = context
        .status
        .write()
        .map_err(|_| anyhow::Error::msg("Failed to lock job status"))?;
    w.progress_items = Some([current, total]);
    w.progress_percent = if total > 0 {
        current as f32 / total as f32
    } else {
        0.0
    };
    if should_cancel(cancel) {
        w.status = "Cancelled".to_string();
        return Err(anyhow::Error::msg("Cancelled"));
    } else {
        w.status = status;
    }
    drop(w);
    context.waker.wake_by_ref();
    Ok(())
}

pub fn should_cancel(rx: &Receiver<()>) -> bool {
    match rx.try_recv() {
        Ok(_) | Err(TryRecvError::Disconnected) => true,
        Err(_) => false,
    }
}
