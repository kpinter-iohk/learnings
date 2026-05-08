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
    pub fn id(&self) -> JobId {
        self.id
    }

    pub fn payload(&self) -> &str {
        &self.payload
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
        assert!(max_attempts >= 1, "max_attempts must be >= 1");
        Queue {
            max_attempts,
            base_delay,
            next_id: 0,
            jobs: HashMap::new(),
        }
    }

    /// Enqueues a new job in `Pending` state and returns its id.
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
        id
    }

    /// Returns the next runnable job, transitioning it to `Running`.
    pub fn checkout(&mut self, now: Instant) -> Option<CheckedOut> {
        let mut chosen: Option<JobId> = None;
        for (id, job) in self.jobs.iter() {
            let runnable = match job.state {
                JobState::Pending => true,
                JobState::FailedPendingRetry => {
                    job.retry_at.map_or(false, |t| t <= now)
                }
                _ => false,
            };
            if runnable {
                match chosen {
                    None => chosen = Some(*id),
                    Some(cur) if id.0 < cur.0 => chosen = Some(*id),
                    _ => {}
                }
            }
        }
        let id = chosen?;
        let job = self.jobs.get_mut(&id).expect("chosen id must exist");
        job.state = JobState::Running;
        Some(CheckedOut {
            id,
            payload: job.payload.clone(),
        })
    }

    /// Marks a `Running` job as `Succeeded`.
    pub fn succeed(&mut self, id: JobId) -> Result<(), QueueError> {
        let job = self.jobs.get_mut(&id).ok_or(QueueError::UnknownJob)?;
        if job.state != JobState::Running {
            return Err(QueueError::NotRunning);
        }
        job.state = JobState::Succeeded;
        job.retry_at = None;
        Ok(())
    }

    /// Marks a `Running` job as failed, scheduling a retry or marking it `Dead`.
    pub fn fail(&mut self, id: JobId, now: Instant) -> Result<(), QueueError> {
        let max_attempts = self.max_attempts;
        let base_delay = self.base_delay;
        let job = self.jobs.get_mut(&id).ok_or(QueueError::UnknownJob)?;
        if job.state != JobState::Running {
            return Err(QueueError::NotRunning);
        }
        job.attempts += 1;
        if job.attempts < max_attempts {
            let exp = job.attempts - 1;
            let factor = 2u32.checked_pow(exp).unwrap_or(u32::MAX);
            let delay = base_delay.saturating_mul(factor);
            job.retry_at = Some(now + delay);
            job.state = JobState::FailedPendingRetry;
        } else {
            job.retry_at = None;
            job.state = JobState::Dead;
        }
        Ok(())
    }

    /// Returns the current state of the job, or `None` for an unknown id.
    pub fn get_state(&self, id: JobId) -> Option<JobState> {
        self.jobs.get(&id).map(|j| j.state)
    }
}
