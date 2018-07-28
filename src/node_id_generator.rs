use std::time::UNIX_EPOCH;

use LocalNodeId;

/// This trait allows for generating the identifiers of the local nodes that belong to a [`Service`].
///
/// [`Service`]: ../service/struct.Service.html
pub trait GenerateLocalNodeId {
    /// Generates an identifier that will be assigned to a new local node that belongs to a [`Service`].
    ///
    /// If the resulting identifier conflicts with the identifier of an alive node,
    /// the [`Service`] will recall this method until it generates an unique identifier.
    ///
    /// [`Service`]: ../service/struct.Service.html
    fn generate_local_node_id(&mut self) -> LocalNodeId;
}

/// An implementation of [`GenerateLocalNodeId`] that generates serial number identifiers.
///
/// [`GenerateLocalNodeId`]: ./trait.GenerateLocalNodeId.html
#[derive(Debug, Default)]
pub struct SerialLocalNodeIdGenerator {
    next_id: u64,
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
            next_id: start_number,
        }
    }
}
impl GenerateLocalNodeId for SerialLocalNodeIdGenerator {
    fn generate_local_node_id(&mut self) -> LocalNodeId {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        LocalNodeId::new(id)
    }
}

/// An implementation of [`GenerateLocalNodeId`] that generates identifiers based on UNIX time in nanoseconds.
///
/// [`GenerateLocalNodeId`]: ./trait.GenerateLocalNodeId.html
#[derive(Debug, Default)]
pub struct UnixtimeLocalNodeIdGenerator;
impl GenerateLocalNodeId for UnixtimeLocalNodeIdGenerator {
    fn generate_local_node_id(&mut self) -> LocalNodeId {
        match UNIX_EPOCH.elapsed() {
            Err(e) => panic!("{}", e),
            Ok(d) => {
                let id = d.as_secs() * 1_000_000_000 + u64::from(d.subsec_nanos());
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
        let mut generator = SerialLocalNodeIdGenerator::new();
        assert_eq!(generator.generate_local_node_id().value(), 0);
        assert_eq!(generator.generate_local_node_id().value(), 1);
        assert_eq!(generator.generate_local_node_id().value(), 2);

        let mut generator = SerialLocalNodeIdGenerator::with_offset(std::u64::MAX);
        assert_eq!(generator.generate_local_node_id().value(), std::u64::MAX);
        assert_eq!(generator.generate_local_node_id().value(), 0);
        assert_eq!(generator.generate_local_node_id().value(), 1);
    }

    #[test]
    fn unixtime_id_generator_works() {
        let mut generator = UnixtimeLocalNodeIdGenerator;
        let id0 = generator.generate_local_node_id();
        std::thread::sleep(std::time::Duration::from_millis(1));
        let id1 = generator.generate_local_node_id();
        assert_ne!(id0, id1);
    }
}
