//! A message broadcasting library based on the Plumtree/HyParView algorithms.
//!
//! # Properties
//!
//! ## Pros
//!
//! - Nearly optimal message transmitting count
//!   - Usually messages are broadcasted via a spanning tree
//!   - Only the nodes interested in the same messages belong to the same cluster
//! - Scalable
//!   - Theoretically, it can handle ten-thousand of nodes or more
//! - High fault tolerance
//!   - Spanning trees are automatically repaired if there are crashed nodes
//! - Dynamic membership
//!   - Nodes can be added to (removed from) a cluster at any time
//!
//! ## Cons
//!
//! - No strong guarantee about connectivity of the nodes in a cluster
//! - No strong guarantee about delivery count of a message
//! - No guarantee about messages delivery order
//!
//! If some of the above guarantees are mandatory for your application,
//! it is need to be provided by upper layers.
//!
//! # References
//!
//! - [HyParView: a membership protocol for reliable gossip-based broadcast][HyParView]
//! - [Plumtree: Epidemic Broadcast Trees][Plumtree]
//!
//! [HyParView]: http://asc.di.fct.unl.pt/~jleitao/pdf/dsn07-leitao.pdf
//! [Plumtree]: http://www.gsd.inesc-id.pt/~ler/reports/srds07.pdf
#![warn(missing_docs)]
extern crate atomic_immut;
#[macro_use]
extern crate bytecodec;
extern crate fibers;
extern crate fibers_rpc;
extern crate futures;
extern crate hyparview;
extern crate plumtree;
extern crate prometrics;
extern crate rand;
#[macro_use]
extern crate slog;
#[macro_use]
extern crate trackable;

pub use clock::Clock;
pub use error::{Error, ErrorKind};
pub use hyparview_misc::{HyparviewNode, HyparviewNodeOptions};
pub use message::{Message, MessageId, MessagePayload};
pub use node::{Node, NodeBuilder};
pub use node_id::{LocalNodeId, NodeId};
pub use plumtree_misc::{PlumtreeNode, PlumtreeNodeOptions, PlumtreeSystem};
pub use service::{Service, ServiceBuilder, ServiceHandle};

mod clock;
mod codec;
mod error;
mod hyparview_misc;
mod message;
mod node;
mod node_id;
mod plumtree_misc;
mod rpc;
mod service;

pub mod metrics;

/// This crate specific `Result` type.
pub type Result<T> = std::result::Result<T, Error>;
