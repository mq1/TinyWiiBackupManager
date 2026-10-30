// SPDX-FileCopyrightText: 2026 Ultimate Game Backup Manager contributors
// SPDX-License-Identifier: GPL-3.0-only

use super::{Progress, SpeedMeter, TaskControl};
use std::{path::PathBuf, time::Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TaskId(u64);

/// Something a task uses that can only serve one task at a time. Tasks that share no resource
/// run side by side; the others wait in the order they were added.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Resource {
    /// A drive or ROM source, by its mount point or root folder: two copies to the same disk
    /// only slow each other down.
    Drive(PathBuf),
    /// A console's connection, by its name in My Consoles (vcinject Link serves one at a time).
    Console(String),
    Network,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskState {
    Waiting,
    Running,
    /// Holds its resources, so nothing else writes to a half-written file meanwhile.
    Paused,
    Done(String),
    Failed(String),
    Cancelled,
}

impl TaskState {
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Running | Self::Paused)
    }

    pub fn is_finished(&self) -> bool {
        matches!(self, Self::Done(_) | Self::Failed(_) | Self::Cancelled)
    }
}

#[derive(Debug, Clone)]
pub struct Task {
    id: TaskId,
    title: String,
    resources: Vec<Resource>,
    state: TaskState,
    progress: Progress,
    control: TaskControl,
    speed: SpeedMeter,
}

impl Task {
    pub fn id(&self) -> TaskId {
        self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn resources(&self) -> &[Resource] {
        &self.resources
    }

    pub fn state(&self) -> &TaskState {
        &self.state
    }

    pub fn progress(&self) -> &Progress {
        &self.progress
    }

    /// The handle the worker doing this task checks for cancel and pause.
    pub fn control(&self) -> TaskControl {
        self.control.clone()
    }
}

/// All tasks, in the order they were added: waiting, running and finished ones (the history).
#[derive(Debug, Default)]
pub struct TaskManager {
    tasks: Vec<Task>,
    next_id: u64,
}

impl TaskManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Task> {
        self.tasks.iter()
    }

    pub fn get(&self, id: TaskId) -> Option<&Task> {
        self.tasks.iter().find(|t| t.id == id)
    }

    fn get_mut(&mut self, id: TaskId) -> Option<&mut Task> {
        self.tasks.iter_mut().find(|t| t.id == id)
    }

    /// Adds a waiting task; [`Self::start_ready`] starts it once its resources are free.
    pub fn add(&mut self, title: impl Into<String>, resources: Vec<Resource>) -> TaskId {
        let id = TaskId(self.next_id);
        self.next_id += 1;

        self.tasks.push(Task {
            id,
            title: title.into(),
            resources,
            state: TaskState::Waiting,
            progress: Progress::default(),
            control: TaskControl::new(),
            speed: SpeedMeter::default(),
        });

        id
    }

    /// Marks every waiting task whose resources are free as running and returns them, first
    /// added first; the caller starts their workers.
    pub fn start_ready(&mut self) -> Vec<TaskId> {
        let mut busy: Vec<Resource> = self
            .tasks
            .iter()
            .filter(|t| t.state.is_active())
            .flat_map(|t| t.resources.iter().cloned())
            .collect();

        let mut started = Vec::new();
        for task in &mut self.tasks {
            if task.state != TaskState::Waiting {
                continue;
            }

            if task.resources.iter().any(|r| busy.contains(r)) {
                // what this task waits for stays reserved, so a later task can't overtake it
                busy.extend(task.resources.iter().cloned());
                continue;
            }

            busy.extend(task.resources.iter().cloned());
            task.state = TaskState::Running;
            started.push(task.id);
        }

        started
    }

    pub fn update_progress(
        &mut self,
        id: TaskId,
        done_bytes: u64,
        total_bytes: Option<u64>,
        message: impl Into<String>,
        now: Instant,
    ) {
        if let Some(task) = self.get_mut(id)
            && task.state.is_active()
        {
            task.progress.speed = task.speed.record(now, done_bytes);
            task.progress.done_bytes = done_bytes;
            task.progress.total_bytes = total_bytes;
            task.progress.message = message.into();
        }
    }

    /// The worker ended: with a message for the user, or the reason it failed. A task that was
    /// cancelled counts as cancelled, whatever the worker returned.
    pub fn finish(&mut self, id: TaskId, result: Result<String, String>) {
        if let Some(task) = self.get_mut(id)
            && task.state.is_active()
        {
            task.state = if task.control.is_cancelled() {
                TaskState::Cancelled
            } else {
                match result {
                    Ok(message) => TaskState::Done(message),
                    Err(reason) => TaskState::Failed(reason),
                }
            };
        }
    }

    /// A waiting task is cancelled at once; a running one when its worker has stopped and
    /// cleaned up (it then calls [`Self::finish`]).
    pub fn cancel(&mut self, id: TaskId) {
        if let Some(task) = self.get_mut(id) {
            match task.state {
                TaskState::Waiting => task.state = TaskState::Cancelled,
                TaskState::Running | TaskState::Paused => {
                    task.control.set_paused(false);
                    task.control.cancel();
                }
                _ => {}
            }
        }
    }

    pub fn pause(&mut self, id: TaskId) {
        if let Some(task) = self.get_mut(id)
            && task.state == TaskState::Running
        {
            task.control.set_paused(true);
            task.state = TaskState::Paused;
        }
    }

    pub fn resume(&mut self, id: TaskId) {
        if let Some(task) = self.get_mut(id)
            && task.state == TaskState::Paused
        {
            task.control.set_paused(false);
            task.state = TaskState::Running;
        }
    }

    /// A failed or cancelled task waits again, at the end of the queue, from the start.
    pub fn retry(&mut self, id: TaskId) {
        let Some(index) = self.tasks.iter().position(|t| t.id == id) else {
            return;
        };

        if matches!(
            self.tasks[index].state,
            TaskState::Failed(_) | TaskState::Cancelled
        ) {
            let mut task = self.tasks.remove(index);
            task.state = TaskState::Waiting;
            task.progress = Progress::default();
            task.control = TaskControl::new();
            task.speed = SpeedMeter::default();
            self.tasks.push(task);
        }
    }

    /// Swaps a waiting task with the waiting task before it (`up`) or after it.
    pub fn move_waiting(&mut self, id: TaskId, up: bool) {
        let waiting: Vec<usize> = (0..self.tasks.len())
            .filter(|&i| self.tasks[i].state == TaskState::Waiting)
            .collect();

        let Some(pos) = waiting.iter().position(|&i| self.tasks[i].id == id) else {
            return;
        };

        let other = if up {
            pos.checked_sub(1)
        } else {
            Some(pos + 1).filter(|&p| p < waiting.len())
        };

        if let Some(other) = other {
            self.tasks.swap(waiting[pos], waiting[other]);
        }
    }

    /// Removes the finished tasks from the history.
    pub fn clear_history(&mut self) {
        self.tasks.retain(|t| !t.state.is_finished());
    }

    pub fn running_count(&self) -> usize {
        self.tasks.iter().filter(|t| t.state.is_active()).count()
    }

    pub fn waiting_count(&self) -> usize {
        self.tasks
            .iter()
            .filter(|t| t.state == TaskState::Waiting)
            .count()
    }

    /// Overall progress of the running tasks that know their size, 0.0 to 1.0.
    pub fn overall_fraction(&self) -> Option<f32> {
        let (done, total) = self
            .tasks
            .iter()
            .filter(|t| t.state.is_active())
            .filter_map(|t| Some((t.progress.done_bytes, t.progress.total_bytes?)))
            .fold((0u64, 0u64), |(d, t), (done, total)| {
                (d + done.min(total), t + total)
            });

        (total > 0).then(|| (done as f64 / total as f64) as f32)
    }

    /// Whether data moves right now, so the computer should stay awake.
    pub fn is_busy(&self) -> bool {
        self.tasks.iter().any(|t| t.state == TaskState::Running)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn drive(name: &str) -> Resource {
        Resource::Drive(PathBuf::from(name))
    }

    fn state(m: &TaskManager, id: TaskId) -> TaskState {
        m.get(id).unwrap().state().clone()
    }

    #[test]
    fn one_task_per_resource_others_in_parallel() {
        let mut m = TaskManager::new();
        let a = m.add("copy A to WiiGames", vec![drive("/Volumes/WiiGames")]);
        let b = m.add("copy B to WiiGames", vec![drive("/Volumes/WiiGames")]);
        let c = m.add(
            "install on petter1",
            vec![Resource::Console("petter1".into())],
        );

        assert_eq!(m.start_ready(), vec![a, c]);
        assert_eq!(state(&m, b), TaskState::Waiting);
        assert!(m.start_ready().is_empty());

        m.finish(a, Ok("done".into()));
        assert_eq!(m.start_ready(), vec![b]);
    }

    #[test]
    fn a_waiting_task_is_not_overtaken() {
        let mut m = TaskManager::new();
        let a = m.add("A", vec![drive("x")]);
        let b = m.add("B", vec![drive("x"), drive("y")]);
        let c = m.add("C", vec![drive("y")]);

        assert_eq!(m.start_ready(), vec![a]);
        // C would be free, but B waits for y too and came first
        assert_eq!(state(&m, c), TaskState::Waiting);

        m.finish(a, Ok(String::new()));
        assert_eq!(m.start_ready(), vec![b]);
        m.finish(b, Ok(String::new()));
        assert_eq!(m.start_ready(), vec![c]);
    }

    #[test]
    fn cancel_pause_and_retry() {
        let mut m = TaskManager::new();
        let a = m.add("A", vec![drive("x")]);
        let b = m.add("B", vec![drive("x")]);
        m.start_ready();

        m.cancel(b);
        assert_eq!(state(&m, b), TaskState::Cancelled);

        m.pause(a);
        assert_eq!(state(&m, a), TaskState::Paused);
        assert!(m.get(a).unwrap().control().is_paused());
        assert!(!m.is_busy());
        m.resume(a);
        assert!(m.is_busy());

        let control = m.get(a).unwrap().control();
        m.cancel(a);
        assert!(control.is_cancelled());
        assert_eq!(state(&m, a), TaskState::Running);
        m.finish(a, Err("Operation interrupted".into()));
        assert_eq!(state(&m, a), TaskState::Cancelled);

        m.retry(a);
        assert_eq!(state(&m, a), TaskState::Waiting);
        assert!(!m.get(a).unwrap().control().is_cancelled());
        assert_eq!(m.start_ready(), vec![a]);
        m.finish(a, Err("Read-only file system".into()));
        assert_eq!(
            state(&m, a),
            TaskState::Failed("Read-only file system".into())
        );

        m.clear_history();
        assert_eq!(m.iter().count(), 0);
    }

    #[test]
    fn reorder_waiting_tasks() {
        let mut m = TaskManager::new();
        let running = m.add("running", vec![drive("x")]);
        let a = m.add("A", vec![drive("x")]);
        let b = m.add("B", vec![drive("x")]);
        m.start_ready();

        m.move_waiting(b, true);
        let order: Vec<TaskId> = m.iter().map(Task::id).collect();
        assert_eq!(order, vec![running, b, a]);

        m.move_waiting(b, true); // already first among the waiting ones
        m.finish(running, Ok(String::new()));
        assert_eq!(m.start_ready(), vec![b]);
    }

    #[test]
    fn progress_and_overall() {
        let mut m = TaskManager::new();
        let a = m.add("A", vec![drive("x")]);
        let b = m.add("B", vec![drive("y")]);
        m.start_ready();

        let t0 = Instant::now();
        m.update_progress(a, 0, Some(100), "Copying", t0);
        m.update_progress(a, 50, Some(100), "Copying", t0 + Duration::from_secs(1));
        m.update_progress(b, 150, Some(300), "Copying", t0);

        let p = m.get(a).unwrap().progress();
        assert_eq!(p.fraction(), Some(0.5));
        assert_eq!(p.time_left(), Some(Duration::from_secs(1)));
        assert_eq!(m.overall_fraction(), Some(0.5));
        assert_eq!(m.running_count(), 2);
        assert_eq!(m.waiting_count(), 0);
    }
}
