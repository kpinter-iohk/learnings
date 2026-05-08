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
    UnknownJob(JobId),
    NotRunning(JobId),
}

impl fmt::Display for QueueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QueueError::UnknownJob(id) => write!(f, "unknown job: {:?}", id),
            QueueError::NotRunning(id) => write!(f, "job {:?} is not in Running state", id),
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

enum JobInternalState {
    Pending,
    Running,
    FailedPendingRetry { retry_at: Instant },
    Succeeded,
    Dead,
}

struct Job {
    payload: String,
    attempts: u32,
    state: JobInternalState,
}

pub struct Queue {
    max_attempts: u32,
    base_delay: Duration,
    next_id: u64,
    jobs: HashMap<JobId, Job>,
    order: Vec<JobId>,
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
        self.jobs.insert(
            id,
            Job {
                payload,
                attempts: 0,
                state: JobInternalState::Pending,
            },
        );
        self.order.push(id);
        id
    }

    pub fn checkout(&mut self, now: Instant) -> Option<CheckedOut> {
        for id in &self.order {
            let job = match self.jobs.get(id) {
                Some(j) => j,
                None => continue,
            };
            let runnable = match &job.state {
                JobInternalState::Pending => true,
                JobInternalState::FailedPendingRetry { retry_at } => *retry_at <= now,
                _ => false,
            };
            if runnable {
                let id = *id;
                let job = self.jobs.get_mut(&id).unwrap();
                job.state = JobInternalState::Running;
                return Some(CheckedOut {
                    id,
                    payload: job.payload.clone(),
                });
            }
        }
        None
    }

    pub fn succeed(&mut self, id: JobId) -> Result<(), QueueError> {
        let job = self.jobs.get_mut(&id).ok_or(QueueError::UnknownJob(id))?;
        match job.state {
            JobInternalState::Running => {
                job.state = JobInternalState::Succeeded;
                Ok(())
            }
            _ => Err(QueueError::NotRunning(id)),
        }
    }

    pub fn fail(&mut self, id: JobId, now: Instant) -> Result<(), QueueError> {
        let max_attempts = self.max_attempts;
        let base_delay = self.base_delay;
        let job = self.jobs.get_mut(&id).ok_or(QueueError::UnknownJob(id))?;
        match job.state {
            JobInternalState::Running => {
                job.attempts += 1;
                if job.attempts < max_attempts {
                    let exp = job.attempts - 1;
                    let delay = base_delay * 2u32.pow(exp);
                    job.state = JobInternalState::FailedPendingRetry {
                        retry_at: now + delay,
                    };
                } else {
                    job.state = JobInternalState::Dead;
                }
                Ok(())
            }
            _ => Err(QueueError::NotRunning(id)),
        }
    }

    pub fn get_state(&self, id: JobId) -> Option<JobState> {
        self.jobs.get(&id).map(|job| match job.state {
            JobInternalState::Pending => JobState::Pending,
            JobInternalState::Running => JobState::Running,
            JobInternalState::FailedPendingRetry { .. } => JobState::FailedPendingRetry,
            JobInternalState::Succeeded => JobState::Succeeded,
            JobInternalState::Dead => JobState::Dead,
        })
    }
}
