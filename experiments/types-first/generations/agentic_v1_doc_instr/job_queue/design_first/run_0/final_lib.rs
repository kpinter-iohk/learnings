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
        assert!(max_attempts >= 1, "max_attempts must be >= 1");
        Queue {
            max_attempts,
            base_delay,
            next_id: 0,
            jobs: HashMap::new(),
        }
    }

    /// Enqueue a new job with the given payload, returning the assigned id.
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

    /// Check out the next runnable job, transitioning it to `Running`.
    pub fn checkout(&mut self, now: Instant) -> Option<CheckedOut> {
        let mut best: Option<JobId> = None;
        for (id, job) in self.jobs.iter() {
            let runnable = match job.state {
                JobState::Pending => true,
                JobState::FailedPendingRetry => {
                    job.retry_at.map(|t| t <= now).unwrap_or(false)
                }
                _ => false,
            };
            if runnable {
                match best {
                    None => best = Some(*id),
                    Some(cur) if id.0 < cur.0 => best = Some(*id),
                    _ => {}
                }
            }
        }
        let id = best?;
        let job = self.jobs.get_mut(&id).expect("id came from self.jobs");
        job.state = JobState::Running;
        Some(CheckedOut {
            id,
            payload: job.payload.clone(),
        })
    }

    /// Mark a `Running` job as `Succeeded`.
    pub fn succeed(&mut self, id: JobId) -> Result<(), QueueError> {
        let job = self.jobs.get_mut(&id).ok_or(QueueError::UnknownJob)?;
        if job.state != JobState::Running {
            return Err(QueueError::NotRunning);
        }
        job.state = JobState::Succeeded;
        job.retry_at = None;
        Ok(())
    }

    /// Mark a `Running` job as failed; schedule a retry or mark `Dead`.
    pub fn fail(&mut self, id: JobId, now: Instant) -> Result<(), QueueError> {
        let max_attempts = self.max_attempts;
        let base_delay = self.base_delay;
        let job = self.jobs.get_mut(&id).ok_or(QueueError::UnknownJob)?;
        if job.state != JobState::Running {
            return Err(QueueError::NotRunning);
        }
        job.attempts = job.attempts.saturating_add(1);
        if job.attempts < max_attempts {
            let exp = job.attempts - 1;
            let multiplier = 2u32.checked_pow(exp).unwrap_or(u32::MAX);
            let delay = base_delay
                .checked_mul(multiplier)
                .unwrap_or(Duration::MAX);
            let retry_at = now.checked_add(delay).unwrap_or_else(|| {
                now.checked_add(Duration::from_secs(60 * 60 * 24 * 365 * 10))
                    .unwrap_or(now)
            });
            job.state = JobState::FailedPendingRetry;
            job.retry_at = Some(retry_at);
        } else {
            job.state = JobState::Dead;
            job.retry_at = None;
        }
        Ok(())
    }

    /// Returns the current state of the job, or `None` if no such job exists.
    pub fn get_state(&self, id: JobId) -> Option<JobState> {
        self.jobs.get(&id).map(|j| j.state)
    }
}
