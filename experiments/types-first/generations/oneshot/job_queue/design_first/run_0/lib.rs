use std::collections::HashMap;
use std::fmt;
use std::time::{Duration, Instant};

/// Public state of a job in the queue.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum JobState {
    Pending,
    Running,
    FailedPendingRetry,
    Succeeded,
    Dead,
}

/// Opaque identifier for a job, assigned by the queue at enqueue time.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct JobId(u64);

/// Errors returned by `succeed` and `fail`.
#[derive(Debug)]
pub enum QueueError {
    /// No job exists with the given id.
    UnknownJob,
    /// The job exists but is not in the `Running` state.
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

/// Handle returned by `Queue::checkout`. Owns a snapshot of the payload so
/// that the queue is free to be mutated while the worker is processing.
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

/// Internal record for a single job in the queue.
struct Job {
    payload: String,
    state: JobState,
    attempts: u32,
    retry_at: Option<Instant>,
}

/// In-memory job queue with exponential-backoff retries.
pub struct Queue {
    max_attempts: u32,
    base_delay: Duration,
    next_id: u64,
    jobs: HashMap<JobId, Job>,
    /// Insertion order; used so that `checkout` is FIFO across both fresh
    /// and retry-ready jobs.
    order: Vec<JobId>,
}

impl Queue {
    /// Create a new queue. Panics if `max_attempts < 1`.
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

    /// Insert a new job with the given payload. Returns its assigned id.
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

    /// Find and check out the next runnable job, transitioning it to
    /// `Running`. Returns `None` if nothing is runnable right now.
    pub fn checkout(&mut self, now: Instant) -> Option<CheckedOut> {
        let mut found: Option<JobId> = None;
        for id in &self.order {
            if let Some(job) = self.jobs.get(id) {
                let runnable = match job.state {
                    JobState::Pending => true,
                    JobState::FailedPendingRetry => {
                        job.retry_at.map_or(false, |t| t <= now)
                    }
                    JobState::Running | JobState::Succeeded | JobState::Dead => false,
                };
                if runnable {
                    found = Some(*id);
                    break;
                }
            }
        }

        let id = found?;
        let job = self.jobs.get_mut(&id).expect("found id must exist");
        job.state = JobState::Running;
        let payload = job.payload.clone();
        Some(CheckedOut { id, payload })
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

    /// Mark a running job as failed. Schedules a retry with exponential
    /// backoff if attempts remain, otherwise marks the job `Dead`.
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
            let factor = 2u32.checked_pow(exp).unwrap_or(u32::MAX);
            let delay = base_delay.checked_mul(factor).unwrap_or(Duration::MAX);
            let retry_at = now.checked_add(delay).unwrap_or(now);
            job.state = JobState::FailedPendingRetry;
            job.retry_at = Some(retry_at);
        } else {
            job.state = JobState::Dead;
            job.retry_at = None;
        }
        Ok(())
    }

    /// Return the current state of a job, or `None` for an unknown id.
    pub fn get_state(&self, id: JobId) -> Option<JobState> {
        self.jobs.get(&id).map(|j| j.state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enqueue_then_checkout_succeed() {
        let mut q = Queue::new(3, Duration::from_secs(1));
        let id = q.enqueue("hello".to_string());
        assert_eq!(q.get_state(id), Some(JobState::Pending));

        let now = Instant::now();
        let co = q.checkout(now).expect("should be runnable");
        assert_eq!(co.id(), id);
        assert_eq!(co.payload(), "hello");
        assert_eq!(q.get_state(id), Some(JobState::Running));

        // Cannot check out twice.
        assert!(q.checkout(now).is_none());

        q.succeed(id).unwrap();
        assert_eq!(q.get_state(id), Some(JobState::Succeeded));

        // Double-success is an error.
        assert!(matches!(q.succeed(id), Err(QueueError::NotRunning)));
    }

    #[test]
    fn fail_schedules_retry_with_exponential_backoff() {
        let base = Duration::from_secs(2);
        let mut q = Queue::new(4, base);
        let id = q.enqueue("p".to_string());

        let t0 = Instant::now();

        // Attempt 1.
        let _ = q.checkout(t0).unwrap();
        q.fail(id, t0).unwrap();
        assert_eq!(q.get_state(id), Some(JobState::FailedPendingRetry));

        // Not yet runnable.
        assert!(q.checkout(t0).is_none());
        assert!(q.checkout(t0 + base - Duration::from_millis(1)).is_none());

        // Runnable at t0 + base*1.
        let _ = q.checkout(t0 + base).expect("retry should be ready");
        assert_eq!(q.get_state(id), Some(JobState::Running));

        // Attempt 2 -> next delay is base * 2.
        q.fail(id, t0 + base).unwrap();
        assert!(q.checkout(t0 + base + base).is_none()); // 2*base later -> 3*base mark
        let _ = q
            .checkout(t0 + base + base * 2)
            .expect("second retry ready");

        // Attempt 3 -> next delay is base * 4.
        let t = t0 + base + base * 2;
        q.fail(id, t).unwrap();
        let _ = q
            .checkout(t + base * 4)
            .expect("third retry ready");

        // Final failure -> Dead (max_attempts == 4).
        q.fail(id, t + base * 4).unwrap();
        assert_eq!(q.get_state(id), Some(JobState::Dead));

        // Dead job is not runnable, and ops on it error.
        assert!(q.checkout(t + base * 100).is_none());
        assert!(matches!(q.succeed(id), Err(QueueError::NotRunning)));
        assert!(matches!(q.fail(id, t), Err(QueueError::NotRunning)));
    }

    #[test]
    fn fifo_across_pending_and_retry_ready() {
        let mut q = Queue::new(3, Duration::from_millis(10));
        let a = q.enqueue("a".to_string());
        let b = q.enqueue("b".to_string());

        let t0 = Instant::now();

        // Take A, fail it. B is still Pending.
        let co = q.checkout(t0).unwrap();
        assert_eq!(co.id(), a);
        q.fail(a, t0).unwrap();

        // Now both runnable at t0 + 10ms; insertion order says A first.
        let t = t0 + Duration::from_millis(10);
        let co1 = q.checkout(t).unwrap();
        assert_eq!(co1.id(), a);
        let co2 = q.checkout(t).unwrap();
        assert_eq!(co2.id(), b);
        assert!(q.checkout(t).is_none());
    }

    #[test]
    fn unknown_job_errors() {
        let mut q = Queue::new(2, Duration::from_secs(1));
        let bogus = JobId(9999);
        assert!(q.get_state(bogus).is_none());
        assert!(matches!(q.succeed(bogus), Err(QueueError::UnknownJob)));
        assert!(matches!(
            q.fail(bogus, Instant::now()),
            Err(QueueError::UnknownJob)
        ));
    }

    #[test]
    fn display_for_queue_error() {
        let s1 = format!("{}", QueueError::UnknownJob);
        let s2 = format!("{}", QueueError::NotRunning);
        assert!(!s1.is_empty());
        assert!(!s2.is_empty());
    }
}
