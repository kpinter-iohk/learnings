use std::collections::BTreeMap;
use std::fmt;
use std::time::{Duration, Instant};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum JobState {
    Pending,
    Running,
    FailedPendingRetry,
    Succeeded,
    Dead,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct JobId(u64);

#[derive(Debug)]
pub enum QueueError {
    UnknownJob,
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

struct Job {
    payload: String,
    state: JobState,
    attempts: u32,
    retry_at: Option<Instant>,
}

pub struct Queue {
    max_attempts: u32,
    base_delay: Duration,
    next_id: u64,
    jobs: BTreeMap<JobId, Job>,
}

impl Queue {
    pub fn new(max_attempts: u32, base_delay: Duration) -> Self {
        assert!(max_attempts >= 1, "max_attempts must be >= 1");
        Queue {
            max_attempts,
            base_delay,
            next_id: 0,
            jobs: BTreeMap::new(),
        }
    }

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

    pub fn checkout(&mut self, now: Instant) -> Option<CheckedOut> {
        for (id, job) in self.jobs.iter_mut() {
            let runnable = match job.state {
                JobState::Pending => true,
                JobState::FailedPendingRetry => {
                    job.retry_at.map_or(false, |t| t <= now)
                }
                _ => false,
            };
            if runnable {
                job.state = JobState::Running;
                return Some(CheckedOut {
                    id: *id,
                    payload: job.payload.clone(),
                });
            }
        }
        None
    }

    pub fn succeed(&mut self, id: JobId) -> Result<(), QueueError> {
        let job = self.jobs.get_mut(&id).ok_or(QueueError::UnknownJob)?;
        if job.state != JobState::Running {
            return Err(QueueError::NotRunning);
        }
        job.state = JobState::Succeeded;
        job.retry_at = None;
        Ok(())
    }

    pub fn fail(&mut self, id: JobId, now: Instant) -> Result<(), QueueError> {
        let max_attempts = self.max_attempts;
        let base_delay = self.base_delay;
        let job = self.jobs.get_mut(&id).ok_or(QueueError::UnknownJob)?;
        if job.state != JobState::Running {
            return Err(QueueError::NotRunning);
        }
        job.attempts = job.attempts.saturating_add(1);
        if job.attempts < max_attempts {
            let exponent = job.attempts - 1;
            let multiplier: u32 = if exponent >= 32 {
                u32::MAX
            } else {
                1u32 << exponent
            };
            job.state = JobState::FailedPendingRetry;
            job.retry_at = Some(now + base_delay * multiplier);
        } else {
            job.state = JobState::Dead;
            job.retry_at = None;
        }
        Ok(())
    }

    pub fn get_state(&self, id: JobId) -> Option<JobState> {
        self.jobs.get(&id).map(|j| j.state)
    }
}
