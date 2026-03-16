use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// A per-session concurrency guard that serializes mutations targeting the same tmux session.
///
/// Concurrent mutations on the same session (e.g., two `rename_session` calls, or a
/// `kill_session` racing with `new_window`) can cause race conditions. `SessionLock`
/// maintains a map of session-scoped mutexes so that only one mutation per session
/// executes at a time, while different sessions remain fully concurrent.
///
/// Read operations (e.g., `list_sessions`, `list_windows`, `capture_pane`) should NOT
/// acquire the lock.
#[derive(Debug, Default)]
pub struct SessionLock {
    locks: Mutex<HashMap<String, Arc<Mutex<()>>>>,
}

impl SessionLock {
    /// Create a new `SessionLock`.
    pub fn new() -> Self {
        Self {
            locks: Mutex::new(HashMap::new()),
        }
    }

    /// Acquire a lock for the given session name.
    ///
    /// Returns a [`SessionGuard`] that holds the per-session mutex. The lock is released
    /// when the guard is dropped. If another task already holds the lock for this session,
    /// this call will wait until the lock becomes available.
    pub async fn acquire(&self, session: &str) -> SessionGuard {
        let lock = {
            let mut map = self.locks.lock().await;
            map.entry(session.to_string())
                .or_insert_with(|| Arc::new(Mutex::new(())))
                .clone()
        };
        let guard = lock.lock_owned().await;
        SessionGuard { _guard: guard }
    }

    /// Remove locks for sessions that are no longer held by any task.
    ///
    /// This prevents unbounded growth of the internal map when sessions are created
    /// and destroyed over time. Only removes entries where no other reference exists
    /// (i.e., no task is waiting on or holding the lock).
    pub async fn cleanup(&self) {
        let mut map = self.locks.lock().await;
        map.retain(|_, v| Arc::strong_count(v) > 1);
    }
}

/// RAII guard returned by [`SessionLock::acquire`].
///
/// Holds the per-session mutex lock. The lock is released when this guard is dropped.
#[derive(Debug)]
pub struct SessionGuard {
    _guard: tokio::sync::OwnedMutexGuard<()>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use tokio::time::{Duration, timeout};

    #[tokio::test]
    async fn acquire_and_release() {
        let lock = SessionLock::new();
        let guard = lock.acquire("test-session").await;
        drop(guard);
        // Should be able to re-acquire after dropping
        let _guard = lock.acquire("test-session").await;
    }

    #[tokio::test]
    async fn different_sessions_are_independent() {
        let lock = Arc::new(SessionLock::new());
        let counter = Arc::new(AtomicU32::new(0));

        let lock_a = lock.clone();
        let counter_a = counter.clone();
        let handle_a = tokio::spawn(async move {
            let _guard = lock_a.acquire("session-a").await;
            counter_a.fetch_add(1, Ordering::SeqCst);
            tokio::time::sleep(Duration::from_millis(50)).await;
            counter_a.fetch_add(1, Ordering::SeqCst);
        });

        let lock_b = lock.clone();
        let counter_b = counter.clone();
        let handle_b = tokio::spawn(async move {
            let _guard = lock_b.acquire("session-b").await;
            counter_b.fetch_add(1, Ordering::SeqCst);
            tokio::time::sleep(Duration::from_millis(50)).await;
            counter_b.fetch_add(1, Ordering::SeqCst);
        });

        handle_a.await.unwrap();
        handle_b.await.unwrap();
        assert_eq!(counter.load(Ordering::SeqCst), 4);
    }

    #[tokio::test]
    async fn same_session_is_serialized() {
        let lock = Arc::new(SessionLock::new());
        let order = Arc::new(Mutex::new(Vec::new()));

        // First task acquires the lock and holds it for a bit
        let lock1 = lock.clone();
        let order1 = order.clone();
        let handle1 = tokio::spawn(async move {
            let _guard = lock1.acquire("shared").await;
            order1.lock().await.push("task1-start");
            tokio::time::sleep(Duration::from_millis(50)).await;
            order1.lock().await.push("task1-end");
        });

        // Small delay to ensure task1 acquires first
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Second task tries to acquire the same session lock
        let lock2 = lock.clone();
        let order2 = order.clone();
        let handle2 = tokio::spawn(async move {
            let _guard = lock2.acquire("shared").await;
            order2.lock().await.push("task2-start");
        });

        handle1.await.unwrap();
        handle2.await.unwrap();

        let sequence = order.lock().await;
        // task1 must complete before task2 starts
        assert_eq!(*sequence, vec!["task1-start", "task1-end", "task2-start"]);
    }

    #[tokio::test]
    async fn cleanup_removes_unused_entries() {
        let lock = SessionLock::new();

        // Acquire and release locks for several sessions
        let _g1 = lock.acquire("sess-1").await;
        drop(_g1);
        let _g2 = lock.acquire("sess-2").await;
        drop(_g2);

        // Both entries should exist but be reclaimable
        lock.cleanup().await;
        let map = lock.locks.lock().await;
        assert!(map.is_empty(), "all unused locks should be cleaned up");
    }

    #[tokio::test]
    async fn cleanup_preserves_active_entries() {
        let lock = Arc::new(SessionLock::new());

        // Hold a guard for sess-1
        let _guard = lock.acquire("sess-1").await;

        // Acquire and release sess-2
        let g2 = lock.acquire("sess-2").await;
        drop(g2);

        lock.cleanup().await;

        let map = lock.locks.lock().await;
        assert!(map.contains_key("sess-1"), "active lock should be preserved");
        assert!(
            !map.contains_key("sess-2"),
            "released lock should be cleaned up"
        );
    }

    #[tokio::test]
    async fn default_creates_empty_lock() {
        let lock = SessionLock::default();
        let map = lock.locks.lock().await;
        assert!(map.is_empty());
    }

    #[tokio::test]
    async fn concurrent_acquire_same_session_blocks() {
        let lock = Arc::new(SessionLock::new());

        // Task1 holds the lock
        let lock1 = lock.clone();
        let guard = lock1.acquire("blocking").await;

        // Task2 should block trying to acquire
        let lock2 = lock.clone();
        let handle = tokio::spawn(async move {
            let _guard = lock2.acquire("blocking").await;
        });

        // The task should not complete within a short timeout while guard is held
        let result = timeout(Duration::from_millis(50), &mut Box::pin(async {
            handle.await.unwrap();
        }))
        .await;
        assert!(result.is_err(), "task2 should be blocked by task1's lock");

        // Release the lock so the background task can complete
        drop(guard);
    }
}
