use std::future::Future;
use std::time::Duration;
use tokio::time::{self, Instant, Timeout};

pub(crate) trait FutureExt: Future + Sized {
    fn timeout(self, duration: Duration) -> Timeout<Self> {
        time::timeout(duration, self)
    }

    fn timeout_at(self, instant: Instant) -> Timeout<Self> {
        time::timeout_at(instant, self)
    }
}

impl<F: Future> FutureExt for F {}
