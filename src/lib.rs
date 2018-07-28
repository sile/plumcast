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

pub use error::{Error, ErrorKind};

mod codec;
mod error;
mod node_id;
mod node_id_generator;
mod rpc;

pub mod message;
pub mod metrics;
pub mod misc;
pub mod node;
pub mod service;

/// This crate specific `Result` type.
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use fibers::{Executor, Spawn, ThreadPoolExecutor};
    use futures::{Future, Stream};

    use node::{Node, SerialLocalNodeIdGenerator};
    use service::Service;

    #[test]
    fn it_works() {
        let server_addr = "127.0.0.1:12121".parse().unwrap();
        let mut executor = ThreadPoolExecutor::new().unwrap();
        let service = Service::<String>::new(
            server_addr,
            executor.handle(),
            SerialLocalNodeIdGenerator::new(),
        );
        let service_handle = service.handle();
        executor.spawn(service.map_err(|e| panic!("{}", e)));

        let mut fibers = Vec::new();
        let mut first_node_id = None;
        for i in 0..100 {
            let mut node = Node::new(service_handle.clone());
            if let Some(id) = first_node_id {
                node.join(id);
            } else {
                first_node_id = Some(node.id());
            }
            if i == 99 {
                node.broadcast("hello".to_owned());
            }
            let spawner = executor.handle();
            let fiber = executor.spawn_monitor(
                node.into_future()
                    .map(move |(message, stream)| {
                        spawner.spawn(stream.for_each(|_| Ok(())).map_err(|_| ()));
                        message.map(|m| m.into_payload())
                    })
                    .map_err(|(e, _)| e),
            );
            fibers.push(fiber);
        }

        for fiber in fibers {
            match executor.run_fiber(fiber).unwrap() {
                Err(e) => panic!("{}", e),
                Ok(message) => {
                    assert_eq!(message, Some("hello".to_owned()));
                }
            }
        }
    }
}
