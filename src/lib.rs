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

pub use error::{Error, ErrorKind};
pub use message::{Message, MessageId, MessagePayload};
pub use node::{LocalNodeId, Node, NodeId};
pub use service::{Service, ServiceBuilder, ServiceHandle};

mod codec;
mod error;
mod message;
mod node;
mod rpc;
mod service;

pub mod metrics;

/// This crate specific `Result` type.
pub type Result<T> = std::result::Result<T, Error>;
