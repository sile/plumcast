#[macro_use]
extern crate clap;
extern crate fibers;
extern crate fibers_rpc;
extern crate futures;
extern crate plumcast;
extern crate slog;
extern crate sloggers;
#[macro_use]
extern crate trackable;

use clap::Arg;
use fibers::sync::mpsc;
use fibers::{Executor, Spawn, ThreadPoolExecutor};
use futures::{Async, Future, Poll, Stream};
use plumcast::ServiceBuilder;
use plumcast::{LocalNodeId, Node, NodeBuilder, NodeId};
use sloggers::terminal::{Destination, TerminalLoggerBuilder};
use sloggers::Build;
use std::net::SocketAddr;
use trackable::error::MainError;

fn main() -> Result<(), MainError> {
    let matches = app_from_crate!()
        .arg(Arg::with_name("PORT").index(1).required(true))
        .arg(
            Arg::with_name("CONTACT_SERVER")
                .long("contact-server")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("LOG_LEVEL")
                .long("log-level")
                .takes_value(true)
                .default_value("info")
                .possible_values(&["debug", "info"]),
        )
        .get_matches();
    let log_level = track_any_err!(matches.value_of("LOG_LEVEL").unwrap().parse())?;
    let logger = track!(
        TerminalLoggerBuilder::new()
            .destination(Destination::Stderr)
            .level(log_level)
            .build()
    )?;
    let port = matches.value_of("PORT").unwrap();
    let addr: SocketAddr = track_any_err!(format!("0.0.0.0:{}", port).parse())?;

    let executor = track_any_err!(ThreadPoolExecutor::new())?;
    let service = ServiceBuilder::new(addr)
        .logger(logger.clone())
        .local_node_id_start(0)
        .finish(executor.handle());

    let mut node = NodeBuilder::new().logger(logger).finish(service.handle());
    if let Some(contact) = matches.value_of("CONTACT_SERVER") {
        let contact: SocketAddr = track_any_err!(contact.parse())?;
        node.join(NodeId::new(contact, LocalNodeId::new(0)));
    }

    let (message_tx, message_rx) = mpsc::channel();
    let node = ChatNode {
        inner: node,
        message_rx,
    };
    executor.spawn(service.map_err(|e| panic!("{}", e)));
    executor.spawn(node);

    std::thread::spawn(move || {
        use std::io::BufRead;
        let stdin = std::io::stdin();
        for line in stdin.lock().lines() {
            let line = if let Ok(line) = line {
                line
            } else {
                break;
            };
            if message_tx.send(line).is_err() {
                break;
            }
        }
    });

    track_any_err!(executor.run())?;
    Ok(())
}

struct ChatNode {
    inner: Node<String>,
    message_rx: mpsc::Receiver<String>,
}
impl Future for ChatNode {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let mut did_something = true;
        while did_something {
            did_something = false;

            while let Async::Ready(Some(m)) = track_try_unwrap!(self.inner.poll()) {
                println!("# MESSAGE: {:?}", m);
                did_something = true;
            }
            while let Async::Ready(Some(m)) = self.message_rx.poll().expect("Never fails") {
                self.inner.broadcast(m);
                did_something = true;
            }
        }
        Ok(Async::NotReady)
    }
}
