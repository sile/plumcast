use std::net::SocketAddr;

/// Identifier used for distinguish local nodes in a process.
///
/// An identifier is assigned automatically to a new [`Node`] when it is created.
///
/// [`Node`]: ./struct.Node.html
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LocalNodeId(u64);
impl LocalNodeId {
    /// Returns the value of the identifier.
    pub fn value(&self) -> u64 {
        self.0
    }

    pub(crate) fn new(id: u64) -> Self {
        LocalNodeId(id)
    }
}

/// Identifier used for distinguish nodes in a cluster.
///
/// The identifier of a [`Node`] consists of [`LocalNodeId`] of the node and
/// the socket address of the RPC server used for communicating with the node.
///
/// [`Node`]: ./struct.Node.html
/// [`LocalNodeId`]: ./struct.LocalNodeId.html
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId {
    address: SocketAddr,
    local_id: LocalNodeId,
}
impl NodeId {
    /// Returns the RPC server address part of the identifier.
    pub fn address(&self) -> SocketAddr {
        self.address
    }

    /// Returns the local node identifier part of the identifier.
    pub fn local_id(&self) -> LocalNodeId {
        self.local_id
    }

    pub(crate) fn new(address: SocketAddr, local_id: LocalNodeId) -> Self {
        NodeId { address, local_id }
    }
}
