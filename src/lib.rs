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
