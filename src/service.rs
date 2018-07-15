use atomic_immut::AtomicImmut;
use fibers::sync::mpsc;
use fibers::Spawn;
use fibers_rpc::client::{
    ClientService as RpcClientService, ClientServiceBuilder as RpcClientServiceBuilder,
    ClientServiceHandle as RpcClientServiceHandle,
};
use fibers_rpc::server::{Server as RpcServer, ServerBuilder as RpcServerBuilder};
use futures::{Async, Future, Poll, Stream};
use slog::{Discard, Logger};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use node::{NodeHandle, NodeId, NodeName};
use rpc::RpcMessage;
use {Error, ErrorKind};

type LocalNodes = Arc<AtomicImmut<HashMap<NodeName, NodeHandle>>>;

#[derive(Debug)]
pub struct ServiceBuilder {
    logger: Logger,
    server_addr: SocketAddr,
    rpc_server_builder: RpcServerBuilder,
    rpc_client_service_builder: RpcClientServiceBuilder,
}
impl ServiceBuilder {
    pub fn new(rpc_server_bind_addr: SocketAddr) -> Self {
        ServiceBuilder {
            logger: Logger::root(Discard, o!()),
            server_addr: rpc_server_bind_addr,
            rpc_server_builder: RpcServerBuilder::new(rpc_server_bind_addr),
            rpc_client_service_builder: RpcClientServiceBuilder::new(),
        }
    }

    pub fn logger(&mut self, logger: Logger) -> &mut Self {
        self.rpc_server_builder.logger(logger.clone());
        self.rpc_client_service_builder.logger(logger.clone());
        self.logger = logger;
        self
    }

    pub fn finish<S>(&mut self, spawner: S) -> Service<S>
    where
        S: Clone + Spawn + Send + 'static,
    {
        let rpc_server = self.rpc_server_builder.finish(spawner.clone());
        let rpc_client_service = self.rpc_client_service_builder.finish(spawner);
        let (command_tx, command_rx) = mpsc::channel();

        Service {
            logger: self.logger.clone(),
            server_addr: self.server_addr,
            command_tx,
            command_rx,
            rpc_server,
            rpc_client_service,
            local_nodes: Default::default(),
        }
    }
}

#[derive(Debug)]
pub struct Service<S> {
    logger: Logger,
    server_addr: SocketAddr,
    command_tx: mpsc::Sender<Command>,
    command_rx: mpsc::Receiver<Command>, // NOTE: infinite stream
    rpc_server: RpcServer<S>,
    rpc_client_service: RpcClientService,
    local_nodes: LocalNodes,
}
impl<S> Service<S>
where
    S: Clone + Spawn + Send + 'static,
{
    pub fn new(rpc_server_bind_addr: SocketAddr, spawner: S) -> Self {
        ServiceBuilder::new(rpc_server_bind_addr).finish(spawner)
    }

    pub fn handle(&self) -> ServiceHandle {
        ServiceHandle {
            server_addr: self.server_addr,
            command_tx: self.command_tx.clone(),
            rpc_client_service: self.rpc_client_service.handle(),
            local_nodes: Arc::clone(&self.local_nodes),
        }
    }

    fn handle_command(&mut self, command: Command) {
        match command {
            Command::Register(node) => {
                info!(self.logger, "Registers a local node: {:?}", node);
                self.local_nodes.update(|nodes| {
                    let mut nodes = (*nodes).clone();
                    nodes.insert(node.name().clone(), node.clone());
                    nodes
                });
            }
            Command::Deregister(node) => {
                info!(self.logger, "Deregisters a local node: {:?}", node);
                self.local_nodes.update(|nodes| {
                    let mut nodes = (*nodes).clone();
                    nodes.remove(&node);
                    nodes
                });
            }
        }
    }
}
impl<S> Future for Service<S>
where
    S: Clone + Spawn + Send + 'static,
{
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if let Async::Ready(()) = track!(self.rpc_client_service.poll())? {
            track_panic!(
                ErrorKind::Other,
                "Unexpected termination of RPC client service"
            );
        }
        if let Async::Ready(()) = track!(self.rpc_server.poll())? {
            track_panic!(ErrorKind::Other, "Unexpected termination of RPC server");
        }
        while let Async::Ready(Some(command)) = self.command_rx.poll().expect("Never fails") {
            self.handle_command(command);
        }
        Ok(Async::NotReady)
    }
}

#[derive(Debug)]
pub struct ServiceHandle {
    pub server_addr: SocketAddr, // TODO
    command_tx: mpsc::Sender<Command>,
    rpc_client_service: RpcClientServiceHandle,
    local_nodes: LocalNodes,
}
impl ServiceHandle {
    pub(crate) fn register_local_node(&self, node: NodeHandle) {
        let command = Command::Register(node);
        let _ = self.command_tx.send(command);
    }

    pub(crate) fn deregister_local_node(&self, node: NodeName) {
        let command = Command::Deregister(node);
        let _ = self.command_tx.send(command);
    }

    pub(crate) fn send_message(&self, peer: NodeId, message: RpcMessage) {
        panic!("{:?}", message);
    }
}

#[derive(Debug)]
enum Command {
    Register(NodeHandle),
    Deregister(NodeName),
}
