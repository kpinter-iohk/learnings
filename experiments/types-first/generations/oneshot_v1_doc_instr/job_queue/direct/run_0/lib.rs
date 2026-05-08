use std::collections::HashMap;
use std::fmt;
use std::time::{Duration, Instant};

/// Unique identifier for a job in the queue. Assigned by the queue at enqueue time.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct JobId(u64);

/// The lifecycle state of a job.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum JobState {
    Pending,
    Running,
    FailedPendingRetry,
    Succeeded,
    Dead,
}

/// Errors returned by queue mutation operations.
#[derive(Debug)]
pub enum QueueError {
    /// The given JobId is not known to the queue.
    UnknownJob,
    /// The targeted job is not in the `Running` state, so the operation is invalid.
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

/// A job that has been checked out of the queue and is now `Running`.
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
    enqueue_seq: u64,
}

/// An in-memory job queue with retry-on-failure and exponential backoff.
pub struct Queue {
    max_attempts: u32,
    base_delay: Duration,
    jobs: HashMap<JobId, Job>,
    next_id: u64,
    next_seq: u64,
}

impl Queue {
    /// Create a new queue. `max_attempts` must be `>= 1`.
    pub fn new(max_attempts: u32, base_delay: Duration) -> Self {
        assert!(max_attempts >= 1, "max_attempts must be >= 1");
        Queue {
            max_attempts,
            base_delay,
            jobs: HashMap::new(),
            next_id: 0,
            next_seq: 0,
        }
    }

    /// Add a new job to the queue in `Pending` state, returning its assigned id.
    pub fn enqueue(&mut self, payload: String) -> JobId {
        let id = JobId(self.next_id);
        self.next_id += 1;
        let seq = self.next_seq;
        self.next_seq += 1;
        self.jobs.insert(
            id,
            Job {
                payload,
                state: JobState::Pending,
                attempts: 0,
                retry_at: None,
                enqueue_seq: seq,
            },
        );
        id
    }

    /// Pick the next runnable job and mark it `Running`. A runnable job is one
    /// in `Pending`, or in `FailedPendingRetry` whose retry time is `<= now`.
    /// Among runnable jobs, the one enqueued earliest is chosen.
    pub fn checkout(&mut self, now: Instant) -> Option<CheckedOut> {
        let mut best: Option<(u64, JobId)> = None;
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
                    None => best = Some((job.enqueue_seq, *id)),
                    Some((s, _)) if job.enqueue_seq < s => {
                        best = Some((job.enqueue_seq, *id))
                    }
                    _ => {}
                }
            }
        }

        let (_, id) = best?;
        let job = self.jobs.get_mut(&id).expect("just looked it up");
        job.state = JobState::Running;
        job.retry_at = None;
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
        Ok(())
    }

    /// Record a failure for a `Running` job. Increments the attempt counter,
    /// then either schedules a retry with exponential backoff, or marks the
    /// job `Dead` if the maximum number of attempts has been reached.
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
            // 2^exp, saturating at u32::MAX for very large exponents.
            let multiplier = 1u32.checked_shl(exp).unwrap_or(u32::MAX);
            let delay = base_delay.saturating_mul(multiplier);
            job.retry_at = Some(now + delay);
            job.state = JobState::FailedPendingRetry;
        } else {
            job.retry_at = None;
            job.state = JobState::Dead;
        }
        Ok(())
    }

    /// Returns the current state of the job, or `None` if the id is unknown.
    pub fn get_state(&self, id: JobId) -> Option<JobState> {
        self.jobs.get(&id).map(|j| j.state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enqueue_and_state() {
        let mut q = Queue::new(3, Duration::from_secs(1));
        let id = q.enqueue("hello".into());
        assert_eq!(q.get_state(id), Some(JobState::Pending));
    }

    #[test]
    fn checkout_returns_pending_then_none() {
        let mut q = Queue::new(3, Duration::from_secs(1));
        let id = q.enqueue("a".into());
        let now = Instant::now();
        let co = q.checkout(now).expect("should have a job");
        assert_eq!(co.id(), id);
        assert_eq!(co.payload(), "a");
        assert_eq!(q.get_state(id), Some(JobState::Running));
        assert!(q.checkout(now).is_none());
    }

    #[test]
    fn fifo_order() {
        let mut q = Queue::new(3, Duration::from_secs(1));
        let a = q.enqueue("a".into());
        let b = q.enqueue("b".into());
        let now = Instant::now();
        assert_eq!(q.checkout(now).unwrap().id(), a);
        assert_eq!(q.checkout(now).unwrap().id(), b);
    }

    #[test]
    fn succeed_transitions() {
        let mut q = Queue::new(3, Duration::from_secs(1));
        let id = q.enqueue("a".into());
        let now = Instant::now();
        q.checkout(now).unwrap();
        q.succeed(id).unwrap();
        assert_eq!(q.get_state(id), Some(JobState::Succeeded));
        assert!(q.succeed(id).is_err());
    }

    #[test]
    fn fail_schedules_retry_with_backoff() {
        let mut q = Queue::new(3, Duration::from_secs(1));
        let id = q.enqueue("a".into());
        let t0 = Instant::now();
        q.checkout(t0).unwrap();
        q.fail(id, t0).unwrap();
        assert_eq!(q.get_state(id), Some(JobState::FailedPendingRetry));
        // Not yet runnable.
        assert!(q.checkout(t0).is_none());
        // After base_delay (2^0 = 1 * 1s), runnable again.
        let t1 = t0 + Duration::from_secs(1);
        let co = q.checkout(t1).expect("should be runnable now");
        assert_eq!(co.id(), id);

        // Second failure -> delay should be 2 * base_delay.
        q.fail(id, t1).unwrap();
        assert!(q.checkout(t1 + Duration::from_secs(1)).is_none());
        assert!(q.checkout(t1 + Duration::from_secs(2)).is_some());
    }

    #[test]
    fn fail_to_dead() {
        let mut q = Queue::new(1, Duration::from_secs(1));
        let id = q.enqueue("a".into());
        let now = Instant::now();
        q.checkout(now).unwrap();
        q.fail(id, now).unwrap();
        assert_eq!(q.get_state(id), Some(JobState::Dead));
        assert!(q.fail(id, now).is_err());
    }

    #[test]
    fn unknown_id() {
        let q = Queue::new(1, Duration::from_secs(1));
        assert!(q.get_state(JobId(999)).is_none());
    }
}
