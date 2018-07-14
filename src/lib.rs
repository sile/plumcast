extern crate atomic_immut;
extern crate bytecodec;
extern crate fibers;
extern crate fibers_rpc;
extern crate futures;
extern crate hyparview;
extern crate plumtree;
extern crate protobuf_codec;
#[macro_use]
extern crate slog;
#[macro_use]
extern crate trackable;

pub use error::{Error, ErrorKind};
pub use node::Node;
pub use service::{Service, ServiceBuilder, ServiceHandle};

mod error;
mod node;
mod service;

pub type Result<T> = std::result::Result<T, Error>;
