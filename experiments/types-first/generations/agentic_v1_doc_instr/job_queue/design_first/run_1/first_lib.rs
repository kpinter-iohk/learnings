use std::collections::HashMap;
use std::fmt;
use std::time::{Duration, Instant};

/// Unique identifier for a job, assigned by the queue at enqueue time.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct JobId(u64);

/// Lifecycle state of a job.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum JobState {
    Pending,
    Running,
    FailedPendingRetry,
    Succeeded,
    Dead,
}

/// Error returned by queue operations.
#[derive(Debug)]
pub enum QueueError {
    /// No job with the given id exists.
    UnknownJob,
    /// The job exists but is not currently `Running`.
    NotRunning,
}

impl fmt::Display for QueueError {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

/// A handle to a job that has been checked out for execution.
pub struct CheckedOut {
    id: JobId,
    payload: String,
}

impl CheckedOut {
    pub fn id(&self) -> JobId {
        todo!()
    }

    pub fn payload(&self) -> &str {
        todo!()
    }
}

/// Internal record stored for every enqueued job.
struct Job {
    payload: String,
    state: JobState,
    attempts: u32,
    /// Earliest time this job is eligible to run again (only meaningful
    /// when the job is in `FailedPendingRetry`).
    retry_at: Option<Instant>,
}

/// In-memory job queue with retry-on-failure.
pub struct Queue {
    max_attempts: u32,
    base_delay: Duration,
    next_id: u64,
    jobs: HashMap<JobId, Job>,
}

impl Queue {
    /// Creates a new queue. `max_attempts` must be `>= 1`.
    pub fn new(max_attempts: u32, base_delay: Duration) -> Self {
        let _ = (max_attempts, base_delay);
        todo!()
    }

    /// Enqueues a new job in `Pending` state and returns its id.
    pub fn enqueue(&mut self, payload: String) -> JobId {
        let _ = payload;
        todo!()
    }

    /// Returns the next runnable job, transitioning it to `Running`.
    pub fn checkout(&mut self, now: Instant) -> Option<CheckedOut> {
        let _ = now;
        todo!()
    }

    /// Marks a `Running` job as `Succeeded`.
    pub fn succeed(&mut self, id: JobId) -> Result<(), QueueError> {
        let _ = id;
        todo!()
    }

    /// Marks a `Running` job as failed, scheduling a retry or marking it `Dead`.
    pub fn fail(&mut self, id: JobId, now: Instant) -> Result<(), QueueError> {
        let _ = (id, now);
        todo!()
    }

    /// Returns the current state of the job, or `None` for an unknown id.
    pub fn get_state(&self, id: JobId) -> Option<JobState> {
        let _ = id;
        todo!()
    }
}
