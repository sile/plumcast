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
use fibers::{Executor, InPlaceExecutor, Spawn};
use futures::{Future, Stream};
use plumcast::ServiceBuilder;
use plumcast::{Node, NodeId, NodeName};
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
            Arg::with_name("GROUP")
                .long("group")
                .takes_value(true)
                .default_value("foo"),
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
    let group = NodeName::new(matches.value_of("GROUP").unwrap());

    let executor = track_any_err!(InPlaceExecutor::new())?;
    let service = ServiceBuilder::new(addr)
        .logger(logger.clone())
        .finish(executor.handle());
    let mut node = Node::new(logger, group.clone(), service.handle());
    if let Some(contact) = matches.value_of("CONTACT_SERVER") {
        let contact: SocketAddr = track_any_err!(contact.parse())?;
        node.join(NodeId {
            name: group,
            addr: contact,
        });
    }
    executor.spawn(service.map_err(|e| panic!("{}", e)));
    executor.spawn(node.for_each(|m| {
        println!("# MESSAGE: {:?}", m);
        Ok(())
    }).map_err(|e| panic!("{}", e)));

    track_any_err!(executor.run())?;
    Ok(())
}
