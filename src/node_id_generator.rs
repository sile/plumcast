use crate::node::LocalNodeId;
use std::fmt;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::UNIX_EPOCH;

/// This trait allows for generating the identifiers of the local nodes that belong to a [`Service`].
///
/// [`Service`]: ../service/struct.Service.html
pub trait GenerateLocalNodeId: Send + Sync + 'static {
    /// Generates an identifier that will be assigned to a new local node that belongs to a [`Service`].
    ///
    /// [`Service`]: ../service/struct.Service.html
    fn generate_local_node_id(&self) -> LocalNodeId;
}

#[derive(Clone)]
pub(crate) struct ArcLocalNodeIdGenerator(Arc<GenerateLocalNodeId>);
impl ArcLocalNodeIdGenerator {
    pub(crate) fn new<T: GenerateLocalNodeId>(inner: T) -> Self {
        ArcLocalNodeIdGenerator(Arc::new(inner))
    }
}
impl fmt::Debug for ArcLocalNodeIdGenerator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ArcLocalNodeIdGenerator(_)")
    }
}
impl GenerateLocalNodeId for ArcLocalNodeIdGenerator {
    fn generate_local_node_id(&self) -> LocalNodeId {
        self.0.generate_local_node_id()
    }
}

/// An implementation of [`GenerateLocalNodeId`] that generates serial number identifiers.
///
/// [`GenerateLocalNodeId`]: ./trait.GenerateLocalNodeId.html
#[derive(Debug, Default)]
pub struct SerialLocalNodeIdGenerator {
    next_id: AtomicUsize,
}
impl SerialLocalNodeIdGenerator {
    /// Makes a new `SerialLocalNodeIdGenerator` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use plumcast::node::{GenerateLocalNodeId, SerialLocalNodeIdGenerator};
    ///
    /// let mut generator = SerialLocalNodeIdGenerator::new();
    /// assert_eq!(generator.generate_local_node_id().value(), 0);
    /// assert_eq!(generator.generate_local_node_id().value(), 1);
    /// assert_eq!(generator.generate_local_node_id().value(), 2);
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Makes a new `SerialLocalNodeIdGenerator` instance with the given start number.
    ///
    /// # Examples
    ///
    /// ```
    /// use plumcast::node::{GenerateLocalNodeId, SerialLocalNodeIdGenerator};
    ///
    /// let mut generator = SerialLocalNodeIdGenerator::with_offset(std::u64::MAX);
    /// assert_eq!(generator.generate_local_node_id().value(), std::u64::MAX);
    /// assert_eq!(generator.generate_local_node_id().value(), 0);
    /// assert_eq!(generator.generate_local_node_id().value(), 1);
    /// ```
    pub fn with_offset(start_number: u64) -> Self {
        SerialLocalNodeIdGenerator {
            next_id: AtomicUsize::new(start_number as usize),
        }
    }
}
impl GenerateLocalNodeId for SerialLocalNodeIdGenerator {
    fn generate_local_node_id(&self) -> LocalNodeId {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        LocalNodeId::new(id as u64)
    }
}

/// An implementation of [`GenerateLocalNodeId`] that generates identifiers based on UNIX time in nanoseconds.
///
/// [`GenerateLocalNodeId`]: ./trait.GenerateLocalNodeId.html
#[derive(Debug, Default)]
pub struct UnixtimeLocalNodeIdGenerator {
    nanos: AtomicUsize,
}
impl UnixtimeLocalNodeIdGenerator {
    /// Makes a new `UnixtimeLocalNodeIdGenerator` instance.
    pub fn new() -> Self {
        Self::default()
    }
}
impl GenerateLocalNodeId for UnixtimeLocalNodeIdGenerator {
    fn generate_local_node_id(&self) -> LocalNodeId {
        match UNIX_EPOCH.elapsed() {
            Err(e) => panic!("{}", e),
            Ok(d) => {
                let nanos = self.nanos.fetch_add(1, Ordering::SeqCst) as u64 % 1_000;
                let micros = u64::from(d.subsec_micros()) * 1_000;
                let id = d.as_secs() * 1_000_000_000 + micros + nanos;
                LocalNodeId::new(id)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std;

    use super::*;

    #[test]
    fn serial_id_generator_works() {
        let generator = SerialLocalNodeIdGenerator::new();
        assert_eq!(generator.generate_local_node_id().value(), 0);
        assert_eq!(generator.generate_local_node_id().value(), 1);
        assert_eq!(generator.generate_local_node_id().value(), 2);

        let generator = SerialLocalNodeIdGenerator::with_offset(std::u64::MAX);
        assert_eq!(generator.generate_local_node_id().value(), std::u64::MAX);
        assert_eq!(generator.generate_local_node_id().value(), 0);
        assert_eq!(generator.generate_local_node_id().value(), 1);
    }

    #[test]
    fn unixtime_id_generator_works() {
        let generator = UnixtimeLocalNodeIdGenerator::new();
        let id0 = generator.generate_local_node_id();
        std::thread::sleep(std::time::Duration::from_millis(1));
        let id1 = generator.generate_local_node_id();
        assert_ne!(id0, id1);
    }
}
