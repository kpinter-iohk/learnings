use std::collections::HashMap;
use std::fmt;
use std::time::{Duration, Instant};

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct JobId(u64);

#[derive(Debug, PartialEq, Eq)]
pub enum JobState {
    Pending,
    Running,
    FailedPendingRetry,
    Succeeded,
    Dead,
}

#[derive(Debug)]
pub enum QueueError {
    UnknownJob,
    NotRunning,
}

impl fmt::Display for QueueError {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

enum JobStatus {
    Pending,
    Running,
    FailedPendingRetry { retry_at: Instant },
    Succeeded,
    Dead,
}

struct Job {
    id: JobId,
    payload: String,
    attempts: u32,
    status: JobStatus,
}

pub struct Queue {
    max_attempts: u32,
    base_delay: Duration,
    next_id: u64,
    jobs: HashMap<JobId, Job>,
    order: Vec<JobId>,
}

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

impl Queue {
    pub fn new(max_attempts: u32, base_delay: Duration) -> Self {
        todo!()
    }

    pub fn enqueue(&mut self, payload: String) -> JobId {
        todo!()
    }

    pub fn checkout(&mut self, now: Instant) -> Option<CheckedOut> {
        todo!()
    }

    pub fn succeed(&mut self, id: JobId) -> Result<(), QueueError> {
        todo!()
    }

    pub fn fail(&mut self, id: JobId, now: Instant) -> Result<(), QueueError> {
        todo!()
    }

    pub fn get_state(&self, id: JobId) -> Option<JobState> {
        todo!()
    }
}
