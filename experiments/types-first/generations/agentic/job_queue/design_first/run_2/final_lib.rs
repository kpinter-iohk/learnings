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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QueueError::UnknownJob => write!(f, "unknown job id"),
            QueueError::NotRunning => write!(f, "job is not in Running state"),
        }
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
        self.id
    }

    pub fn payload(&self) -> &str {
        &self.payload
    }
}

impl Queue {
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

    pub fn enqueue(&mut self, payload: String) -> JobId {
        let id = JobId(self.next_id);
        self.next_id += 1;
        let job = Job {
            payload,
            attempts: 0,
            status: JobStatus::Pending,
        };
        self.jobs.insert(id, job);
        self.order.push(id);
        id
    }

    pub fn checkout(&mut self, now: Instant) -> Option<CheckedOut> {
        let mut chosen: Option<JobId> = None;
        for id in &self.order {
            let job = match self.jobs.get(id) {
                Some(j) => j,
                None => continue,
            };
            match &job.status {
                JobStatus::Pending => {
                    chosen = Some(*id);
                    break;
                }
                JobStatus::FailedPendingRetry { retry_at } if *retry_at <= now => {
                    chosen = Some(*id);
                    break;
                }
                _ => {}
            }
        }
        let id = chosen?;
        let job = self.jobs.get_mut(&id).expect("job exists");
        job.status = JobStatus::Running;
        Some(CheckedOut {
            id,
            payload: job.payload.clone(),
        })
    }

    pub fn succeed(&mut self, id: JobId) -> Result<(), QueueError> {
        let job = self.jobs.get_mut(&id).ok_or(QueueError::UnknownJob)?;
        match job.status {
            JobStatus::Running => {
                job.status = JobStatus::Succeeded;
                Ok(())
            }
            _ => Err(QueueError::NotRunning),
        }
    }

    pub fn fail(&mut self, id: JobId, now: Instant) -> Result<(), QueueError> {
        let max_attempts = self.max_attempts;
        let base_delay = self.base_delay;
        let job = self.jobs.get_mut(&id).ok_or(QueueError::UnknownJob)?;
        match job.status {
            JobStatus::Running => {
                job.attempts += 1;
                if job.attempts < max_attempts {
                    let exp = job.attempts - 1;
                    let multiplier: u32 = 1u32 << exp;
                    let delay = base_delay * multiplier;
                    job.status = JobStatus::FailedPendingRetry {
                        retry_at: now + delay,
                    };
                } else {
                    job.status = JobStatus::Dead;
                }
                Ok(())
            }
            _ => Err(QueueError::NotRunning),
        }
    }

    pub fn get_state(&self, id: JobId) -> Option<JobState> {
        let job = self.jobs.get(&id)?;
        Some(match job.status {
            JobStatus::Pending => JobState::Pending,
            JobStatus::Running => JobState::Running,
            JobStatus::FailedPendingRetry { .. } => JobState::FailedPendingRetry,
            JobStatus::Succeeded => JobState::Succeeded,
            JobStatus::Dead => JobState::Dead,
        })
    }
}
