//! Task-spawning shims that route through dial9 when the `dial9` feature is
//! enabled, so spawned tasks get wake-event tracking in the trace. Without the
//! feature these are plain `tokio` spawns.

use std::future::Future;

use tokio::task::{AbortHandle, JoinHandle, JoinSet};

#[track_caller]
pub fn spawn<F>(future: F) -> JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    #[cfg(feature = "dial9")]
    {
        dial9::spawn(future)
    }
    #[cfg(not(feature = "dial9"))]
    {
        tokio::spawn(future)
    }
}

#[track_caller]
pub fn spawn_in_set<T, F>(set: &mut JoinSet<T>, future: F) -> AbortHandle
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    #[cfg(feature = "dial9")]
    {
        use dial9::JoinSetExt;
        set.spawn_traced(future)
    }
    #[cfg(not(feature = "dial9"))]
    {
        set.spawn(future)
    }
}
