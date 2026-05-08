use std::collections::HashMap;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct JobId(u64);

#[derive(Debug, Clone, Copy)]
pub enum QueueError {
    UnknownJob(JobId),
    NotRunning(JobId, JobState),
}

impl fmt::Display for QueueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QueueError::UnknownJob(id) => {
                write!(f, "unknown job id: {:?}", id)
            }
            QueueError::NotRunning(id, state) => {
                write!(
                    f,
                    "job {:?} is not in Running state (current state: {:?})",
                    id, state
                )
            }
        }
    }
}

impl std::error::Error for QueueError {}

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
    id: JobId,
    payload: String,
    state: JobState,
    attempts: u32,
    retry_at: Option<Instant>,
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
        let job = Job {
            id,
            payload,
            state: JobState::Pending,
            attempts: 0,
            retry_at: None,
        };
        self.jobs.insert(id, job);
        self.order.push(id);
        id
    }

    pub fn checkout(&mut self, now: Instant) -> Option<CheckedOut> {
        let mut chosen: Option<JobId> = None;
        for &id in &self.order {
            let job = match self.jobs.get(&id) {
                Some(j) => j,
                None => continue,
            };
            match job.state {
                JobState::Pending => {
                    chosen = Some(id);
                    break;
                }
                JobState::FailedPendingRetry => {
                    if let Some(retry_at) = job.retry_at {
                        if retry_at <= now {
                            chosen = Some(id);
                            break;
                        }
                    }
                }
                _ => {}
            }
        }

        let id = chosen?;
        let job = self.jobs.get_mut(&id).expect("job exists");
        job.state = JobState::Running;
        Some(CheckedOut {
            id,
            payload: job.payload.clone(),
        })
    }

    pub fn succeed(&mut self, id: JobId) -> Result<(), QueueError> {
        let job = self
            .jobs
            .get_mut(&id)
            .ok_or(QueueError::UnknownJob(id))?;
        if job.state != JobState::Running {
            return Err(QueueError::NotRunning(id, job.state));
        }
        job.state = JobState::Succeeded;
        job.retry_at = None;
        Ok(())
    }

    pub fn fail(&mut self, id: JobId, now: Instant) -> Result<(), QueueError> {
        let max_attempts = self.max_attempts;
        let base_delay = self.base_delay;
        let job = self
            .jobs
            .get_mut(&id)
            .ok_or(QueueError::UnknownJob(id))?;
        if job.state != JobState::Running {
            return Err(QueueError::NotRunning(id, job.state));
        }
        job.attempts = job.attempts.saturating_add(1);
        let new_attempts = job.attempts;
        if new_attempts < max_attempts {
            let exp = new_attempts - 1;
            let multiplier: u32 = if exp >= 32 {
                u32::MAX
            } else {
                1u32 << exp
            };
            let delay = base_delay
                .checked_mul(multiplier)
                .unwrap_or(Duration::MAX);
            let retry_at = now.checked_add(delay).unwrap_or_else(|| {
                // Saturate to a far-future-ish instant by adding the
                // largest amount we can. If even that fails, fall back to now.
                now.checked_add(Duration::from_secs(u64::MAX / 2))
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

    pub fn get_state(&self, id: JobId) -> Option<JobState> {
        self.jobs.get(&id).map(|j| j.state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enqueue_and_checkout_pending() {
        let mut q = Queue::new(3, Duration::from_secs(1));
        let id = q.enqueue("hello".to_string());
        assert_eq!(q.get_state(id), Some(JobState::Pending));
        let now = Instant::now();
        let co = q.checkout(now).expect("should check out");
        assert_eq!(co.id(), id);
        assert_eq!(co.payload(), "hello");
        assert_eq!(q.get_state(id), Some(JobState::Running));
        // Cannot be checked out again.
        assert!(q.checkout(now).is_none());
    }

    #[test]
    fn succeed_running_job() {
        let mut q = Queue::new(3, Duration::from_secs(1));
        let id = q.enqueue("p".to_string());
        let now = Instant::now();
        let _co = q.checkout(now).unwrap();
        q.succeed(id).unwrap();
        assert_eq!(q.get_state(id), Some(JobState::Succeeded));
        assert!(q.succeed(id).is_err());
    }

    #[test]
    fn fail_retries_then_dies() {
        let mut q = Queue::new(3, Duration::from_secs(1));
        let id = q.enqueue("p".to_string());
        let t0 = Instant::now();

        // Attempt 1: fail -> attempts=1, < 3, retry at t0 + 1s
        let _ = q.checkout(t0).unwrap();
        q.fail(id, t0).unwrap();
        assert_eq!(q.get_state(id), Some(JobState::FailedPendingRetry));
        // Not yet runnable
        assert!(q.checkout(t0).is_none());
        let t1 = t0 + Duration::from_secs(1);
        let co = q.checkout(t1).unwrap();
        assert_eq!(co.id(), id);

        // Attempt 2: fail -> attempts=2, < 3, retry at t1 + 2s
        q.fail(id, t1).unwrap();
        assert_eq!(q.get_state(id), Some(JobState::FailedPendingRetry));
        assert!(q.checkout(t1 + Duration::from_secs(1)).is_none());
        let t2 = t1 + Duration::from_secs(2);
        let _co = q.checkout(t2).unwrap();

        // Attempt 3: fail -> attempts=3, not < 3, Dead
        q.fail(id, t2).unwrap();
        assert_eq!(q.get_state(id), Some(JobState::Dead));
        assert!(q.checkout(t2 + Duration::from_secs(3600)).is_none());
    }

    #[test]
    fn errors_for_unknown_and_non_running() {
        let mut q = Queue::new(2, Duration::from_secs(1));
        let bogus = JobId(9999);
        assert!(matches!(q.succeed(bogus), Err(QueueError::UnknownJob(_))));
        assert!(matches!(
            q.fail(bogus, Instant::now()),
            Err(QueueError::UnknownJob(_))
        ));

        let id = q.enqueue("p".to_string());
        // Pending -> can't succeed/fail
        assert!(matches!(
            q.succeed(id),
            Err(QueueError::NotRunning(_, JobState::Pending))
        ));
        assert!(matches!(
            q.fail(id, Instant::now()),
            Err(QueueError::NotRunning(_, JobState::Pending))
        ));
    }

    #[test]
    fn fifo_among_runnable_jobs() {
        let mut q = Queue::new(3, Duration::from_secs(1));
        let a = q.enqueue("a".to_string());
        let b = q.enqueue("b".to_string());
        let now = Instant::now();
        let co1 = q.checkout(now).unwrap();
        assert_eq!(co1.id(), a);
        let co2 = q.checkout(now).unwrap();
        assert_eq!(co2.id(), b);
        assert!(q.checkout(now).is_none());
    }

    #[test]
    fn display_for_queue_error() {
        let e = QueueError::UnknownJob(JobId(7));
        let _ = format!("{}", e);
        let e2 = QueueError::NotRunning(JobId(7), JobState::Pending);
        let _ = format!("{}", e2);
    }
}
