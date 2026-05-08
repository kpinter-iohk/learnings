//! In-memory job queue with retry-on-failure.

use std::collections::HashMap;
use std::fmt;
use std::time::{Duration, Instant};

/// Unique identifier for a job in the queue.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct JobId(u64);

/// State of a job in the queue.
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum JobState {
    Pending,
    Running,
    FailedPendingRetry,
    Succeeded,
    Dead,
}

/// Errors returned by [`Queue`] operations.
#[derive(Debug)]
pub enum QueueError {
    /// The given job id is unknown to the queue.
    UnknownJob,
    /// The job is not currently in the `Running` state.
    NotRunning,
}

impl fmt::Display for QueueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

/// Handle returned from [`Queue::checkout`], exposing job id and payload.
pub struct CheckedOut {
    id: JobId,
    payload: String,
}

impl CheckedOut {
    /// The id of the checked-out job.
    pub fn id(&self) -> JobId {
        todo!()
    }

    /// The payload of the checked-out job.
    pub fn payload(&self) -> &str {
        todo!()
    }
}

/// Internal record of a job's full state.
struct Job {
    payload: String,
    state: JobState,
    attempts: u32,
    retry_at: Option<Instant>,
}

/// In-memory job queue with bounded retries and exponential backoff.
pub struct Queue {
    max_attempts: u32,
    base_delay: Duration,
    next_id: u64,
    jobs: HashMap<JobId, Job>,
    order: Vec<JobId>,
}

impl Queue {
    /// Create a new empty queue.
    ///
    /// `max_attempts` must be `>= 1`. After this many failures a job becomes `Dead`.
    /// `base_delay` is the unit of exponential backoff: the i-th retry (1-indexed)
    /// is scheduled for `now + base_delay * 2^(i-1)`.
    pub fn new(max_attempts: u32, base_delay: Duration) -> Self {
        todo!()
    }

    /// Enqueue a new job carrying the given payload, returning its assigned id.
    pub fn enqueue(&mut self, payload: String) -> JobId {
        todo!()
    }

    /// Check out the next runnable job, transitioning it to `Running`.
    ///
    /// A runnable job is one in state `Pending`, or one in
    /// `FailedPendingRetry` whose retry time is `<= now`.
    pub fn checkout(&mut self, now: Instant) -> Option<CheckedOut> {
        todo!()
    }

    /// Mark a running job as succeeded.
    pub fn succeed(&mut self, id: JobId) -> Result<(), QueueError> {
        todo!()
    }

    /// Mark a running job as failed.
    ///
    /// Increments the attempt counter; if attempts < `max_attempts`, the job
    /// becomes `FailedPendingRetry` with retry time
    /// `now + base_delay * 2^(attempts - 1)`. Otherwise it becomes `Dead`.
    pub fn fail(&mut self, id: JobId, now: Instant) -> Result<(), QueueError> {
        todo!()
    }

    /// Look up a job's current state by id.
    pub fn get_state(&self, id: JobId) -> Option<JobState> {
        todo!()
    }
}
