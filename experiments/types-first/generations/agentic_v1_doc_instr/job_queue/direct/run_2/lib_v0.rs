use std::collections::HashMap;
use std::fmt;
use std::time::{Duration, Instant};

#[derive(Debug, PartialEq, Eq)]
pub enum JobState {
    Pending,
    Running,
    FailedPendingRetry,
    Succeeded,
    Dead,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct JobId(u64);

#[derive(Debug)]
pub enum QueueError {
    UnknownId,
    NotRunning,
}

impl fmt::Display for QueueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QueueError::UnknownId => write!(f, "unknown job id"),
            QueueError::NotRunning => write!(f, "job is not in Running state"),
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

enum InternalState {
    Pending,
    Running,
    FailedPendingRetry { retry_at: Instant },
    Succeeded,
    Dead,
}

struct Job {
    payload: String,
    state: InternalState,
    attempts: u32,
    seq: u64,
}

pub struct Queue {
    max_attempts: u32,
    base_delay: Duration,
    next_id: u64,
    next_seq: u64,
    jobs: HashMap<JobId, Job>,
}

impl Queue {
    pub fn new(max_attempts: u32, base_delay: Duration) -> Self {
        assert!(max_attempts >= 1, "max_attempts must be >= 1");
        Queue {
            max_attempts,
            base_delay,
            next_id: 0,
            next_seq: 0,
            jobs: HashMap::new(),
        }
    }

    pub fn enqueue(&mut self, payload: String) -> JobId {
        let id = JobId(self.next_id);
        self.next_id += 1;
        let seq = self.next_seq;
        self.next_seq += 1;
        self.jobs.insert(
            id,
            Job {
                payload,
                state: InternalState::Pending,
                attempts: 0,
                seq,
            },
        );
        id
    }

    pub fn checkout(&mut self, now: Instant) -> Option<CheckedOut> {
        let mut best: Option<(JobId, u64)> = None;
        for (id, job) in self.jobs.iter() {
            let runnable = match &job.state {
                InternalState::Pending => true,
                InternalState::FailedPendingRetry { retry_at } => *retry_at <= now,
                _ => false,
            };
            if runnable {
                match best {
                    None => best = Some((*id, job.seq)),
                    Some((_, best_seq)) if job.seq < best_seq => best = Some((*id, job.seq)),
                    _ => {}
                }
            }
        }
        let (id, _) = best?;
        let job = self.jobs.get_mut(&id).expect("job exists");
        job.state = InternalState::Running;
        Some(CheckedOut {
            id,
            payload: job.payload.clone(),
        })
    }

    pub fn succeed(&mut self, id: JobId) -> Result<(), QueueError> {
        let job = self.jobs.get_mut(&id).ok_or(QueueError::UnknownId)?;
        match job.state {
            InternalState::Running => {
                job.state = InternalState::Succeeded;
                Ok(())
            }
            _ => Err(QueueError::NotRunning),
        }
    }

    pub fn fail(&mut self, id: JobId, now: Instant) -> Result<(), QueueError> {
        let job = self.jobs.get_mut(&id).ok_or(QueueError::UnknownId)?;
        match job.state {
            InternalState::Running => {
                job.attempts += 1;
                if job.attempts < self.max_attempts {
                    let exponent = job.attempts - 1;
                    let multiplier = 1u32 << exponent;
                    let delay = self.base_delay * multiplier;
                    job.state = InternalState::FailedPendingRetry {
                        retry_at: now + delay,
                    };
                } else {
                    job.state = InternalState::Dead;
                }
                Ok(())
            }
            _ => Err(QueueError::NotRunning),
        }
    }

    pub fn get_state(&self, id: JobId) -> Option<JobState> {
        self.jobs.get(&id).map(|job| match job.state {
            InternalState::Pending => JobState::Pending,
            InternalState::Running => JobState::Running,
            InternalState::FailedPendingRetry { .. } => JobState::FailedPendingRetry,
            InternalState::Succeeded => JobState::Succeeded,
            InternalState::Dead => JobState::Dead,
        })
    }
}
