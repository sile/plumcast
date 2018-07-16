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
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use node::{LocalNodeId, NodeHandle, NodeId};
use rpc::{self, RpcMessage};
use {Error, ErrorKind};

type LocalNodes = Arc<AtomicImmut<HashMap<LocalNodeId, NodeHandle>>>;

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
        let (command_tx, command_rx) = mpsc::channel();
        let rpc_client_service = self.rpc_client_service_builder.finish(spawner.clone());
        let handle = ServiceHandle {
            server_addr: self.server_addr,
            command_tx: command_tx.clone(),
            rpc_client_service: rpc_client_service.handle(),
            local_nodes: Default::default(),
            next_local_id: Arc::new(AtomicUsize::new(0)),
        };

        rpc::hyparview::register_handlers(&mut self.rpc_server_builder, handle.clone());
        rpc::plumtree::register_handlers(&mut self.rpc_server_builder, handle.clone());
        let rpc_server = self.rpc_server_builder.finish(spawner);

        Service {
            logger: self.logger.clone(),
            command_rx,
            rpc_server,
            rpc_client_service,
            handle,
        }
    }
}

#[derive(Debug)]
pub struct Service<S> {
    logger: Logger,
    command_rx: mpsc::Receiver<Command>, // NOTE: infinite stream
    rpc_server: RpcServer<S>,
    rpc_client_service: RpcClientService,
    handle: ServiceHandle,
}
impl<S> Service<S>
where
    S: Clone + Spawn + Send + 'static,
{
    pub fn new(rpc_server_bind_addr: SocketAddr, spawner: S) -> Self {
        ServiceBuilder::new(rpc_server_bind_addr).finish(spawner)
    }

    pub fn handle(&self) -> ServiceHandle {
        self.handle.clone()
    }

    fn handle_command(&mut self, command: Command) {
        match command {
            Command::Register(node) => {
                info!(self.logger, "Registers a local node: {:?}", node);
                self.handle.local_nodes.update(|nodes| {
                    let mut nodes = (*nodes).clone();
                    nodes.insert(node.local_id(), node.clone());
                    nodes
                });
            }
            Command::Deregister(node) => {
                info!(self.logger, "Deregisters a local node: {:?}", node);
                self.handle.local_nodes.update(|nodes| {
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

#[derive(Debug, Clone)]
pub struct ServiceHandle {
    server_addr: SocketAddr,
    command_tx: mpsc::Sender<Command>,
    rpc_client_service: RpcClientServiceHandle,
    local_nodes: LocalNodes,
    next_local_id: Arc<AtomicUsize>,
}
impl ServiceHandle {
    pub(crate) fn generate_node_id(&self) -> NodeId {
        let local_id = LocalNodeId::new(self.next_local_id.fetch_add(1, Ordering::SeqCst) as u64);
        NodeId {
            addr: self.server_addr,
            local_id,
        }
    }

    pub(crate) fn get_local_node(&self, local_id: &LocalNodeId) -> Option<NodeHandle> {
        self.local_nodes.load().get(local_id).cloned()
    }

    pub(crate) fn register_local_node(&self, node: NodeHandle) {
        let command = Command::Register(node);
        let _ = self.command_tx.send(command);
    }

    pub(crate) fn deregister_local_node(&self, node: LocalNodeId) {
        let command = Command::Deregister(node);
        let _ = self.command_tx.send(command);
    }

    pub(crate) fn send_message(&self, peer: NodeId, message: RpcMessage) {
        match message {
            RpcMessage::Hyparview(m) => {
                use hyparview::message::ProtocolMessage;
                match m {
                    ProtocolMessage::Join(m) => {
                        rpc::hyparview::join_cast(peer, m, &self.rpc_client_service);
                    }
                    ProtocolMessage::ForwardJoin(m) => {
                        rpc::hyparview::forward_join_cast(peer, m, &self.rpc_client_service);
                    }
                    ProtocolMessage::Neighbor(m) => {
                        rpc::hyparview::neighbor_cast(peer, m, &self.rpc_client_service);
                    }
                    ProtocolMessage::Shuffle(m) => {
                        rpc::hyparview::shuffle_cast(peer, m, &self.rpc_client_service);
                    }
                    ProtocolMessage::ShuffleReply(m) => {
                        rpc::hyparview::shuffle_reply_cast(peer, m, &self.rpc_client_service);
                    }
                    ProtocolMessage::Disconnect(m) => {
                        rpc::hyparview::disconnect_cast(peer, m, &self.rpc_client_service);
                    }
                }
            }
            RpcMessage::Plumtree(m) => {
                use plumtree::message::ProtocolMessage;
                match m {
                    ProtocolMessage::Gossip(m) => {
                        rpc::plumtree::gossip_cast(peer, m, &self.rpc_client_service);
                    }
                    ProtocolMessage::Ihave(m) => {
                        rpc::plumtree::ihave_cast(peer, m, &self.rpc_client_service);
                    }
                    ProtocolMessage::Graft(m) => {
                        rpc::plumtree::graft_cast(peer, m, &self.rpc_client_service);
                    }
                    ProtocolMessage::Prune(m) => {
                        rpc::plumtree::prune_cast(peer, m, &self.rpc_client_service);
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
enum Command {
    Register(NodeHandle),
    Deregister(LocalNodeId),
}
