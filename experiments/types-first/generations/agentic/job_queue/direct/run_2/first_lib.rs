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

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
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

struct Job {
    payload: String,
    attempts: u32,
    state: JobState,
    retry_at: Option<Instant>,
    enqueue_seq: u64,
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
                attempts: 0,
                state: JobState::Pending,
                retry_at: None,
                enqueue_seq: seq,
            },
        );
        id
    }

    pub fn checkout(&mut self, now: Instant) -> Option<CheckedOut> {
        let mut best: Option<(JobId, u64)> = None;
        for (id, job) in self.jobs.iter() {
            let runnable = match job.state {
                JobState::Pending => true,
                JobState::FailedPendingRetry => {
                    job.retry_at.map_or(false, |t| t <= now)
                }
                _ => false,
            };
            if runnable {
                match best {
                    None => best = Some((*id, job.enqueue_seq)),
                    Some((_, seq)) if job.enqueue_seq < seq => {
                        best = Some((*id, job.enqueue_seq));
                    }
                    _ => {}
                }
            }
        }
        let (id, _) = best?;
        let job = self.jobs.get_mut(&id)?;
        job.state = JobState::Running;
        job.retry_at = None;
        Some(CheckedOut {
            id,
            payload: job.payload.clone(),
        })
    }

    pub fn succeed(&mut self, id: JobId) -> Result<(), QueueError> {
        let job = self.jobs.get_mut(&id).ok_or(QueueError::UnknownJob)?;
        if job.state != JobState::Running {
            return Err(QueueError::NotRunning);
        }
        job.state = JobState::Succeeded;
        Ok(())
    }

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
            let delay = base_delay.saturating_mul(1u32 << exp.min(31));
            job.state = JobState::FailedPendingRetry;
            job.retry_at = Some(now + delay);
        } else {
            job.state = JobState::Dead;
            job.retry_at = None;
        }
        Ok(())
    }

    pub fn get_state(&self, id: JobId) -> Option<JobState> {
        self.jobs.get(&id).map(|j| match j.state {
            JobState::Pending => JobState::Pending,
            JobState::Running => JobState::Running,
            JobState::FailedPendingRetry => JobState::FailedPendingRetry,
            JobState::Succeeded => JobState::Succeeded,
            JobState::Dead => JobState::Dead,
        })
    }
}
