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
        match self {
            QueueError::UnknownJob => f.write_str("unknown job id"),
            QueueError::NotRunning => f.write_str("job is not in the Running state"),
        }
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
        self.id
    }

    /// The payload of the checked-out job.
    pub fn payload(&self) -> &str {
        &self.payload
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
        assert!(max_attempts >= 1, "max_attempts must be >= 1");
        Queue {
            max_attempts,
            base_delay,
            next_id: 0,
            jobs: HashMap::new(),
            order: Vec::new(),
        }
    }

    /// Enqueue a new job carrying the given payload, returning its assigned id.
    pub fn enqueue(&mut self, payload: String) -> JobId {
        let id = JobId(self.next_id);
        self.next_id += 1;
        self.jobs.insert(
            id,
            Job {
                payload,
                state: JobState::Pending,
                attempts: 0,
                retry_at: None,
            },
        );
        self.order.push(id);
        id
    }

    /// Check out the next runnable job, transitioning it to `Running`.
    ///
    /// A runnable job is one in state `Pending`, or one in
    /// `FailedPendingRetry` whose retry time is `<= now`.
    pub fn checkout(&mut self, now: Instant) -> Option<CheckedOut> {
        for id in &self.order {
            let job = match self.jobs.get(id) {
                Some(j) => j,
                None => continue,
            };
            let runnable = match job.state {
                JobState::Pending => true,
                JobState::FailedPendingRetry => {
                    job.retry_at.map_or(false, |t| t <= now)
                }
                _ => false,
            };
            if runnable {
                let id = *id;
                let job = self.jobs.get_mut(&id).expect("job present");
                job.state = JobState::Running;
                return Some(CheckedOut {
                    id,
                    payload: job.payload.clone(),
                });
            }
        }
        None
    }

    /// Mark a running job as succeeded.
    pub fn succeed(&mut self, id: JobId) -> Result<(), QueueError> {
        let job = self.jobs.get_mut(&id).ok_or(QueueError::UnknownJob)?;
        if job.state != JobState::Running {
            return Err(QueueError::NotRunning);
        }
        job.state = JobState::Succeeded;
        job.retry_at = None;
        Ok(())
    }

    /// Mark a running job as failed.
    ///
    /// Increments the attempt counter; if attempts < `max_attempts`, the job
    /// becomes `FailedPendingRetry` with retry time
    /// `now + base_delay * 2^(attempts - 1)`. Otherwise it becomes `Dead`.
    pub fn fail(&mut self, id: JobId, now: Instant) -> Result<(), QueueError> {
        let max_attempts = self.max_attempts;
        let base_delay = self.base_delay;
        let job = self.jobs.get_mut(&id).ok_or(QueueError::UnknownJob)?;
        if job.state != JobState::Running {
            return Err(QueueError::NotRunning);
        }
        job.attempts += 1;
        if job.attempts < max_attempts {
            let factor: u32 = 1u32 << (job.attempts - 1);
            job.retry_at = Some(now + base_delay * factor);
            job.state = JobState::FailedPendingRetry;
        } else {
            job.retry_at = None;
            job.state = JobState::Dead;
        }
        Ok(())
    }

    /// Look up a job's current state by id.
    pub fn get_state(&self, id: JobId) -> Option<JobState> {
        self.jobs.get(&id).map(|j| j.state.clone())
    }
}
