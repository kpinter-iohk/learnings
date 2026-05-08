use std::collections::HashMap;
use std::fmt;
use std::time::{Duration, Instant};

/// Unique identifier for a job assigned by the queue at enqueue time.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct JobId(u64);

/// State of a job in the queue.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum JobState {
    Pending,
    Running,
    FailedPendingRetry,
    Succeeded,
    Dead,
}

/// Errors returned by queue operations.
#[derive(Debug)]
pub enum QueueError {
    /// No job with the given id exists.
    UnknownJob,
    /// The job is not currently `Running`.
    NotRunning,
}

impl fmt::Display for QueueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QueueError::UnknownJob => write!(f, "unknown job id"),
            QueueError::NotRunning => write!(f, "job is not currently running"),
        }
    }
}

/// A handle to a job that has been checked out for execution.
pub struct CheckedOut {
    id: JobId,
    payload: String,
}

impl CheckedOut {
    /// The id of the checked-out job.
    pub fn id(&self) -> JobId {
        self.id
    }

    /// The payload of the checked-out job.
    pub fn payload(&self) -> &str {
        &self.payload
    }
}

/// Internal record for a single job stored in the queue.
struct Job {
    payload: String,
    state: JobState,
    attempts: u32,
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
    /// Construct a new queue. `max_attempts` must be `>= 1`.
    pub fn new(max_attempts: u32, base_delay: Duration) -> Self {
        let _ = (max_attempts, base_delay);
        todo!()
    }

    /// Enqueue a new job with the given payload, returning the assigned id.
    pub fn enqueue(&mut self, payload: String) -> JobId {
        let _ = payload;
        todo!()
    }

    /// Check out the next runnable job, transitioning it to `Running`.
    pub fn checkout(&mut self, now: Instant) -> Option<CheckedOut> {
        let _ = now;
        todo!()
    }

    /// Mark a `Running` job as `Succeeded`.
    pub fn succeed(&mut self, id: JobId) -> Result<(), QueueError> {
        let _ = id;
        todo!()
    }

    /// Mark a `Running` job as failed; schedule a retry or mark `Dead`.
    pub fn fail(&mut self, id: JobId, now: Instant) -> Result<(), QueueError> {
        let _ = (id, now);
        todo!()
    }

    /// Returns the current state of the job, or `None` if no such job exists.
    pub fn get_state(&self, id: JobId) -> Option<JobState> {
        let _ = id;
        todo!()
    }
}
