//! Miscellaneous components.
use fibers::Spawn;
use futures::Future;
use std::fmt;
use std::sync::Arc;

type ArcFn = Arc<Fn(Box<Future<Item = (), Error = ()> + Send>) + Send + Sync + 'static>;

/// Sharable [`Spawn`].
///
/// [`Spawn`]: https://docs.rs/fibers/0.1/fibers/trait.Spawn.html
#[derive(Clone)]
pub struct ArcSpawn(ArcFn);
impl ArcSpawn {
    pub(crate) fn new<S>(inner: S) -> Self
    where
        S: Spawn + Send + Sync + 'static,
    {
        ArcSpawn(Arc::new(move |fiber| inner.spawn_boxed(fiber)))
    }
}
impl Spawn for ArcSpawn {
    fn spawn_boxed(&self, fiber: Box<Future<Item = (), Error = ()> + Send>) {
        (self.0)(fiber);
    }
}
impl fmt::Debug for ArcSpawn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ArcSpawn(_)")
    }
}
