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

use node::{LocalNodeId, MessagePayload, NodeHandle, NodeId};
use rpc::{self, RpcMessage};
use {Error, ErrorKind, Result};

type LocalNodes<M> = Arc<AtomicImmut<HashMap<LocalNodeId, NodeHandle<M>>>>;

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

    pub fn finish<S, M>(&mut self, spawner: S) -> Service<S, M>
    where
        S: Clone + Spawn + Send + 'static,
        M: MessagePayload,
    {
        let (command_tx, command_rx) = mpsc::channel();
        let rpc_client_service = self.rpc_client_service_builder.finish(spawner.clone());
        let handle = ServiceHandle {
            server_addr: self.server_addr,
            command_tx: command_tx.clone(),
            rpc_service: rpc_client_service.handle(),
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
pub struct Service<S, M: MessagePayload> {
    logger: Logger,
    command_rx: mpsc::Receiver<Command<M>>, // NOTE: infinite stream
    rpc_server: RpcServer<S>,
    rpc_client_service: RpcClientService,
    handle: ServiceHandle<M>,
}
impl<S, M> Service<S, M>
where
    S: Clone + Spawn + Send + 'static,
    M: MessagePayload,
{
    pub fn new(rpc_server_bind_addr: SocketAddr, spawner: S) -> Self {
        ServiceBuilder::new(rpc_server_bind_addr).finish(spawner)
    }

    pub fn handle(&self) -> ServiceHandle<M> {
        self.handle.clone()
    }

    fn handle_command(&mut self, command: Command<M>) {
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
impl<S, M> Future for Service<S, M>
where
    S: Clone + Spawn + Send + 'static,
    M: MessagePayload,
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
pub struct ServiceHandle<M: MessagePayload> {
    server_addr: SocketAddr,
    command_tx: mpsc::Sender<Command<M>>,
    rpc_service: RpcClientServiceHandle,
    local_nodes: LocalNodes<M>,
    next_local_id: Arc<AtomicUsize>,
}
impl<M: MessagePayload> ServiceHandle<M> {
    pub(crate) fn generate_node_id(&self) -> NodeId {
        let local_id = LocalNodeId::new(self.next_local_id.fetch_add(1, Ordering::SeqCst) as u64);
        NodeId {
            addr: self.server_addr,
            local_id,
        }
    }

    pub(crate) fn get_local_node(&self, local_id: &LocalNodeId) -> Option<NodeHandle<M>> {
        self.local_nodes.load().get(local_id).cloned()
    }

    pub(crate) fn get_local_node_or_disconnect(
        &self,
        id: &LocalNodeId,
        sender: &NodeId,
    ) -> Option<NodeHandle<M>> {
        if let Some(node) = self.local_nodes.load().get(id).cloned() {
            Some(node)
        } else {
            use hyparview::message::{DisconnectMessage, ProtocolMessage};
            // TODO: metrics
            let missing = NodeId {
                addr: self.server_addr,
                local_id: id.clone(),
            };
            let message = DisconnectMessage { sender: missing };
            let message = ProtocolMessage::Disconnect(message);
            let _ = self.send_message(sender.clone(), RpcMessage::Hyparview(message));
            None
        }
    }

    pub(crate) fn register_local_node(&self, node: NodeHandle<M>) {
        let command = Command::Register(node);
        let _ = self.command_tx.send(command);
    }

    pub(crate) fn deregister_local_node(&self, node: LocalNodeId) {
        let command = Command::Deregister(node);
        let _ = self.command_tx.send(command);
    }

    pub(crate) fn send_message(&self, peer: NodeId, message: RpcMessage<M>) -> Result<()> {
        match message {
            RpcMessage::Hyparview(m) => {
                use hyparview::message::ProtocolMessage;
                use rpc::hyparview;

                match m {
                    ProtocolMessage::Join(m) => {
                        track!(hyparview::join_cast(peer, m, &self.rpc_service))?;
                    }
                    ProtocolMessage::ForwardJoin(m) => {
                        track!(hyparview::forward_join_cast(peer, m, &self.rpc_service))?;
                    }
                    ProtocolMessage::Neighbor(m) => {
                        track!(hyparview::neighbor_cast(peer, m, &self.rpc_service))?;
                    }
                    ProtocolMessage::Shuffle(m) => {
                        track!(hyparview::shuffle_cast(peer, m, &self.rpc_service))?;
                    }
                    ProtocolMessage::ShuffleReply(m) => {
                        track!(hyparview::shuffle_reply_cast(peer, m, &self.rpc_service))?;
                    }
                    ProtocolMessage::Disconnect(m) => {
                        track!(hyparview::disconnect_cast(peer, m, &self.rpc_service))?;
                    }
                }
            }
            RpcMessage::Plumtree(m) => {
                use plumtree::message::ProtocolMessage;
                use rpc::plumtree;

                match m {
                    ProtocolMessage::Gossip(m) => {
                        track!(plumtree::gossip_cast(peer, m, &self.rpc_service))?;
                    }
                    ProtocolMessage::Ihave(m) => {
                        track!(plumtree::ihave_cast(peer, m, &self.rpc_service))?;
                    }
                    ProtocolMessage::Graft(m) => {
                        track!(plumtree::graft_cast(peer, m, &self.rpc_service))?;
                    }
                    ProtocolMessage::Prune(m) => {
                        track!(plumtree::prune_cast(peer, m, &self.rpc_service))?;
                    }
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
enum Command<M: MessagePayload> {
    Register(NodeHandle<M>),
    Deregister(LocalNodeId),
}
