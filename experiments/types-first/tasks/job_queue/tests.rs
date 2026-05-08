use solution::{JobState, Queue};
use std::time::{Duration, Instant};

fn t0() -> Instant {
    Instant::now()
}

#[test]
fn enqueue_returns_unique_ids() {
    let mut q = Queue::new(3, Duration::from_millis(100));
    let a = q.enqueue("a".into());
    let b = q.enqueue("b".into());
    assert_ne!(a, b);
}

#[test]
fn checkout_on_empty_queue_is_none() {
    let mut q = Queue::new(3, Duration::from_millis(100));
    assert!(q.checkout(t0()).is_none());
}

#[test]
fn checkout_returns_enqueued_payload() {
    let mut q = Queue::new(3, Duration::from_millis(100));
    let _ = q.enqueue("hello".into());
    let co = q.checkout(t0()).expect("expected a checkout");
    assert_eq!(co.payload(), "hello");
}

#[test]
fn checkout_transitions_to_running() {
    let mut q = Queue::new(3, Duration::from_millis(100));
    let id = q.enqueue("p".into());
    let co = q.checkout(t0()).expect("checkout");
    assert_eq!(co.id(), id);
    assert_eq!(q.get_state(id), Some(JobState::Running));
}

#[test]
fn no_double_checkout_while_running() {
    let mut q = Queue::new(3, Duration::from_millis(100));
    let _ = q.enqueue("p".into());
    let _co = q.checkout(t0()).expect("first checkout");
    assert!(q.checkout(t0()).is_none());
}

#[test]
fn succeed_transitions_to_succeeded() {
    let mut q = Queue::new(3, Duration::from_millis(100));
    let id = q.enqueue("p".into());
    let _ = q.checkout(t0()).expect("checkout");
    q.succeed(id).expect("succeed");
    assert_eq!(q.get_state(id), Some(JobState::Succeeded));
}

#[test]
fn fail_with_retries_left_pending_retry() {
    let mut q = Queue::new(3, Duration::from_millis(100));
    let id = q.enqueue("p".into());
    let now = t0();
    let _ = q.checkout(now).expect("checkout");
    q.fail(id, now).expect("fail");
    assert_eq!(q.get_state(id), Some(JobState::FailedPendingRetry));
}

#[test]
fn retry_not_returned_before_retry_time() {
    let mut q = Queue::new(3, Duration::from_millis(100));
    let id = q.enqueue("p".into());
    let now = t0();
    let _ = q.checkout(now).expect("checkout");
    q.fail(id, now).expect("fail");
    assert!(q.checkout(now + Duration::from_millis(50)).is_none());
}

#[test]
fn retry_returned_after_retry_time() {
    let mut q = Queue::new(3, Duration::from_millis(100));
    let id = q.enqueue("p".into());
    let now = t0();
    let _ = q.checkout(now).expect("checkout");
    q.fail(id, now).expect("fail");
    let co = q
        .checkout(now + Duration::from_millis(150))
        .expect("expected retry");
    assert_eq!(co.id(), id);
}

#[test]
fn backoff_doubles_on_second_failure() {
    // base = 100ms. First retry at +100ms (2^0). Second retry from second-fail-time +200ms (2^1).
    let mut q = Queue::new(5, Duration::from_millis(100));
    let id = q.enqueue("p".into());
    let t = t0();

    let _ = q.checkout(t).expect("checkout 1");
    q.fail(id, t).expect("fail 1");

    let t1 = t + Duration::from_millis(150);
    let _ = q.checkout(t1).expect("checkout 2");
    q.fail(id, t1).expect("fail 2");

    // After second fail, retry is at t1 + 200ms. At t1 + 150ms it should NOT be ready.
    assert!(q.checkout(t1 + Duration::from_millis(150)).is_none());
    // At t1 + 250ms it SHOULD be ready.
    let co = q
        .checkout(t1 + Duration::from_millis(250))
        .expect("expected second retry");
    assert_eq!(co.id(), id);
}

#[test]
fn exhausted_attempts_transition_to_dead() {
    let mut q = Queue::new(2, Duration::from_millis(10));
    let id = q.enqueue("p".into());
    let mut t = t0();

    let _ = q.checkout(t).expect("checkout 1");
    q.fail(id, t).expect("fail 1");

    t += Duration::from_millis(50);
    let _ = q.checkout(t).expect("checkout 2");
    q.fail(id, t).expect("fail 2");

    assert_eq!(q.get_state(id), Some(JobState::Dead));
}

#[test]
fn dead_jobs_never_checked_out() {
    let mut q = Queue::new(1, Duration::from_millis(10));
    let _id = q.enqueue("p".into());
    let mut t = t0();
    let id = q.checkout(t).expect("checkout 1").id();
    q.fail(id, t).expect("fail 1");
    t += Duration::from_secs(10);
    assert!(q.checkout(t).is_none());
}

#[test]
fn succeed_on_non_running_is_error() {
    let mut q = Queue::new(3, Duration::from_millis(100));
    let id = q.enqueue("p".into());
    assert!(q.succeed(id).is_err());
}

#[test]
fn fail_on_succeeded_is_error() {
    let mut q = Queue::new(3, Duration::from_millis(100));
    let id = q.enqueue("p".into());
    let _ = q.checkout(t0()).expect("checkout");
    q.succeed(id).expect("succeed");
    assert!(q.fail(id, t0()).is_err());
}

#[test]
fn fail_on_pending_is_error() {
    // job is freshly enqueued, never checked out: fail should error
    let mut q = Queue::new(3, Duration::from_millis(100));
    let id = q.enqueue("p".into());
    assert!(q.fail(id, t0()).is_err());
}
