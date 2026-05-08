use std::collections::HashMap;
use std::fmt;
use std::time::{Duration, Instant};

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct JobId(u64);

#[derive(Clone, PartialEq, Eq, Debug)]
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
    jobs: HashMap<JobId, Job>,
    order: Vec<JobId>,
}

impl Queue {
    pub fn new(max_attempts: u32, base_delay: Duration) -> Self {
        assert!(max_attempts >= 1, "max_attempts must be >= 1");
        Self {
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
        for id in &self.order {
            if let Some(job) = self.jobs.get(id) {
                let runnable = match job.state {
                    JobState::Pending => true,
                    JobState::FailedPendingRetry => match job.retry_at {
                        Some(t) => t <= now,
                        None => true,
                    },
                    _ => false,
                };
                if runnable {
                    chosen = Some(*id);
                    break;
                }
            }
        }

        let id = chosen?;
        let job = self.jobs.get_mut(&id).expect("job must exist");
        job.state = JobState::Running;
        let payload = job.payload.clone();
        Some(CheckedOut { id, payload })
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
            let exp = job.attempts - 1;
            let multiplier: u32 = 1u32.checked_shl(exp).unwrap_or(u32::MAX);
            let delay = base_delay.checked_mul(multiplier).unwrap_or(Duration::MAX);
            let retry_at = now.checked_add(delay).unwrap_or_else(|| {
                // Fallback: if adding delay overflows Instant, use a very far future
                // by adding the largest representable delay we can.
                now + Duration::from_secs(u64::MAX / 2)
            });
            job.retry_at = Some(retry_at);
            job.state = JobState::FailedPendingRetry;
        } else {
            job.retry_at = None;
            job.state = JobState::Dead;
        }
        Ok(())
    }

    pub fn get_state(&self, id: JobId) -> Option<JobState> {
        self.jobs.get(&id).map(|j| j.state.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enqueue_and_checkout() {
        let mut q = Queue::new(3, Duration::from_secs(1));
        let id = q.enqueue("hello".to_string());
        assert_eq!(q.get_state(id), Some(JobState::Pending));

        let now = Instant::now();
        let co = q.checkout(now).expect("should get a job");
        assert_eq!(co.id(), id);
        assert_eq!(co.payload(), "hello");
        assert_eq!(q.get_state(id), Some(JobState::Running));

        // Cannot checkout again while running.
        assert!(q.checkout(now).is_none());
    }

    #[test]
    fn succeed_transitions() {
        let mut q = Queue::new(3, Duration::from_secs(1));
        let id = q.enqueue("p".to_string());
        let now = Instant::now();
        let _ = q.checkout(now).unwrap();
        q.succeed(id).unwrap();
        assert_eq!(q.get_state(id), Some(JobState::Succeeded));
        assert!(q.succeed(id).is_err());
    }

    #[test]
    fn fail_with_retry_then_dead() {
        let mut q = Queue::new(3, Duration::from_secs(1));
        let id = q.enqueue("p".to_string());
        let t0 = Instant::now();

        // Attempt 1
        let _ = q.checkout(t0).unwrap();
        q.fail(id, t0).unwrap();
        assert_eq!(q.get_state(id), Some(JobState::FailedPendingRetry));

        // Not yet runnable
        assert!(q.checkout(t0).is_none());

        // After base_delay (1s * 2^0 = 1s)
        let t1 = t0 + Duration::from_secs(1);
        let _ = q.checkout(t1).expect("should be runnable");
        assert_eq!(q.get_state(id), Some(JobState::Running));

        // Attempt 2
        q.fail(id, t1).unwrap();
        assert_eq!(q.get_state(id), Some(JobState::FailedPendingRetry));
        // Delay = 1s * 2^1 = 2s
        assert!(q.checkout(t1 + Duration::from_secs(1)).is_none());
        let t2 = t1 + Duration::from_secs(2);
        let _ = q.checkout(t2).expect("should be runnable");

        // Attempt 3 -> dead
        q.fail(id, t2).unwrap();
        assert_eq!(q.get_state(id), Some(JobState::Dead));
        assert!(q.checkout(t2 + Duration::from_secs(60)).is_none());
    }

    #[test]
    fn fifo_order() {
        let mut q = Queue::new(2, Duration::from_secs(1));
        let a = q.enqueue("a".to_string());
        let b = q.enqueue("b".to_string());
        let now = Instant::now();
        let co = q.checkout(now).unwrap();
        assert_eq!(co.id(), a);
        let co = q.checkout(now).unwrap();
        assert_eq!(co.id(), b);
    }

    #[test]
    fn unknown_id_errors() {
        let mut q = Queue::new(2, Duration::from_secs(1));
        let bogus = JobId(999);
        assert!(matches!(q.succeed(bogus), Err(QueueError::UnknownJob)));
        assert!(matches!(
            q.fail(bogus, Instant::now()),
            Err(QueueError::UnknownJob)
        ));
        assert_eq!(q.get_state(bogus), None);
    }

    #[test]
    fn cannot_succeed_or_fail_pending() {
        let mut q = Queue::new(2, Duration::from_secs(1));
        let id = q.enqueue("p".to_string());
        assert!(matches!(q.succeed(id), Err(QueueError::NotRunning)));
        assert!(matches!(
            q.fail(id, Instant::now()),
            Err(QueueError::NotRunning)
        ));
    }
}
