use fibers::time::timer::{self, Timeout};
use futures::{Async, Future, Poll, Stream};
use std::time::Duration;

use Error;

/// [`Node`] local clock.
///
/// A clock ticks at the intervals specified by the user.
///
/// [`Node`]: ./struct.Node.html
#[derive(Debug)]
pub struct Clock {
    ticks: u64,
    tick_interval: Duration,
    tick_timeout: Timeout,
}
impl Clock {
    /// Returns the elapsed time in ticks since the associated node was created.
    pub fn ticks(&self) -> u64 {
        self.ticks
    }

    /// Returns the tick interval of the clock.
    pub fn tick_interval(&self) -> Duration {
        self.tick_interval
    }

    pub(crate) fn new(tick_interval: Duration) -> Self {
        Clock {
            ticks: 0,
            tick_interval,
            tick_timeout: timer::timeout(tick_interval),
        }
    }
}
impl Stream for Clock {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        if track!(self.tick_timeout.poll().map_err(Error::from))?.is_ready() {
            self.ticks += 1;
            self.tick_timeout = timer::timeout(self.tick_interval);
            Ok(Async::Ready(Some(())))
        } else {
            Ok(Async::NotReady)
        }
    }
}
