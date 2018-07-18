use atomic_immut::AtomicImmut;
use fibers::sync::mpsc;
use fibers::Spawn;
use fibers_rpc::client::{
    ClientService as RpcClientService, ClientServiceBuilder as RpcClientServiceBuilder,
    ClientServiceHandle as RpcClientServiceHandle,
};
use fibers_rpc::server::{Server as RpcServer, ServerBuilder as RpcServerBuilder};
use futures::{Async, Future, Poll, Stream};
use prometrics::metrics::MetricBuilder;
use rand::{self, Rng};
use slog::{Discard, Logger};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use metrics::ServiceMetrics;
use node::NodeHandle;
use rpc::{self, RpcMessage};
use {Error, ErrorKind, LocalNodeId, MessagePayload, NodeId, Result};

type LocalNodes<M> = Arc<AtomicImmut<HashMap<LocalNodeId, NodeHandle<M>>>>;

/// The builder of [`Service`].
///
/// [`Service`]: ./struct.Service.html
#[derive(Debug)]
pub struct ServiceBuilder {
    logger: Logger,
    server_addr: SocketAddr,
    rpc_server_builder: RpcServerBuilder,
    rpc_client_service_builder: RpcClientServiceBuilder,
    metrics: MetricBuilder,
    local_node_id_start: u64,
}
impl ServiceBuilder {
    /// Makes a new `ServiceBuilder` instance with the default settings.
    pub fn new(rpc_server_bind_addr: SocketAddr) -> Self {
        ServiceBuilder {
            logger: Logger::root(Discard, o!()),
            server_addr: rpc_server_bind_addr,
            rpc_server_builder: RpcServerBuilder::new(rpc_server_bind_addr),
            rpc_client_service_builder: RpcClientServiceBuilder::new(),
            metrics: MetricBuilder::new(),
            local_node_id_start: rand::thread_rng().gen(),
        }
    }

    /// Sets the logger used by the service.
    ///
    /// The default value is `Logger::root(Discard, o!())`.
    pub fn logger(mut self, logger: Logger) -> Self {
        self.rpc_server_builder.logger(logger.clone());
        self.rpc_client_service_builder.logger(logger.clone());
        self.logger = logger;
        self
    }

    /// Sets the metrics settings of the service.
    ///
    /// The default value is `MetricBuilder::new()`.
    pub fn metrics(mut self, metrics: MetricBuilder) -> Self {
        self.metrics = metrics;
        self
    }

    /// Sets the value of the identifier of the first local node associated with the service.
    ///
    /// Local node identifiers are increased incrementally start from the specified value.
    ///
    /// The default value is `rand::thread_rng().gen()`.
    pub fn local_node_id_start(mut self, n: u64) -> Self {
        self.local_node_id_start = n;
        self
    }

    /// Builds a [`Service`] with the given settings.
    ///
    /// [`Service`]: ./struct.Service.html
    pub fn finish<S, M>(mut self, spawner: S) -> Service<S, M>
    where
        S: Spawn + Clone + Send + 'static,
        M: MessagePayload,
    {
        let (command_tx, command_rx) = mpsc::channel();
        let rpc_client_service = self.rpc_client_service_builder.finish(spawner.clone());

        let metrics = ServiceMetrics::new(self.metrics);
        let handle = ServiceHandle {
            server_addr: self.server_addr,
            command_tx: command_tx.clone(),
            rpc_service: rpc_client_service.handle(),
            local_nodes: Default::default(),
            next_local_id: Arc::new(AtomicUsize::new(self.local_node_id_start as usize)),
            metrics: metrics.clone(),
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
            metrics,
        }
    }
}

/// A [`Future`] that executes management tasks needed for running a plumcast system.
///
/// This has mainly two responsibilities:
/// - The management of the RPC server and client used for inter node communication by Plumtree/HyParView
/// - [`Node`] registry
///
/// [`Future`]: https://docs.rs/futures/0.1/futures/future/trait.Future.html
/// [`Node`]: ./struct.Node.html
#[derive(Debug)]
#[must_use = "futures do nothing unless polled"]
pub struct Service<S, M: MessagePayload> {
    logger: Logger,
    command_rx: mpsc::Receiver<Command<M>>, // NOTE: infinite stream
    rpc_server: RpcServer<S>,
    rpc_client_service: RpcClientService,
    handle: ServiceHandle<M>,
    metrics: ServiceMetrics,
}
impl<S, M> Service<S, M>
where
    S: Spawn + Clone + Send + 'static,
    M: MessagePayload,
{
    /// Makes a new `Service` instance with the default settings.
    ///
    /// If you want to customize settings, please use [`ServiceBuilder`] instead.
    ///
    /// [`ServiceBuilder`]: ./struct.ServiceBuilder.html
    pub fn new(rpc_server_bind_addr: SocketAddr, spawner: S) -> Self {
        ServiceBuilder::new(rpc_server_bind_addr).finish(spawner)
    }

    /// Returns the handle of the service.
    pub fn handle(&self) -> ServiceHandle<M> {
        self.handle.clone()
    }

    /// Returns a reference to the RPC server of the service.
    pub fn rpc_server(&self) -> &RpcServer<S> {
        &self.rpc_server
    }

    /// Returns a reference to the RPC client service of the service.
    pub fn rpc_client_service(&self) -> &RpcClientService {
        &self.rpc_client_service
    }

    fn handle_command(&mut self, command: Command<M>) -> Result<()> {
        match command {
            Command::Register(node) => {
                info!(self.logger, "Registers a local node: {:?}", node);
                track_assert!(
                    !self.handle
                        .local_nodes
                        .load()
                        .contains_key(&node.local_id()),
                    ErrorKind::InconsistentState; node
                );

                self.metrics.registered_nodes.increment();
                self.handle.local_nodes.update(|nodes| {
                    let mut nodes = (*nodes).clone();
                    nodes.insert(node.local_id(), node.clone());
                    nodes
                });
            }
            Command::Deregister(node) => {
                info!(self.logger, "Deregisters a local node: {:?}", node);
                track_assert!(
                    self.handle.local_nodes.load().contains_key(&node),
                    ErrorKind::InconsistentState; node
                );

                self.metrics.deregistered_nodes.increment();
                self.handle.local_nodes.update(|nodes| {
                    let mut nodes = (*nodes).clone();
                    nodes.remove(&node);
                    nodes
                });
            }
        }
        Ok(())
    }
}
impl<S, M> Future for Service<S, M>
where
    S: Spawn + Clone + Send + 'static,
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
            track!(self.handle_command(command))?;
        }
        Ok(Async::NotReady)
    }
}
impl<S, M: MessagePayload> Drop for Service<S, M> {
    fn drop(&mut self) {
        let old = self.handle.local_nodes.swap(HashMap::new());
        self.metrics.deregistered_nodes.add_u64(old.len() as u64);
    }
}

/// A handle of a [`Service`] instance.
///
/// [`Service`]: ./struct.Service.html
#[derive(Debug, Clone)]
pub struct ServiceHandle<M: MessagePayload> {
    server_addr: SocketAddr,
    command_tx: mpsc::Sender<Command<M>>,
    rpc_service: RpcClientServiceHandle,
    local_nodes: LocalNodes<M>,
    next_local_id: Arc<AtomicUsize>,
    metrics: ServiceMetrics,
}
impl<M: MessagePayload> ServiceHandle<M> {
    /// Returns the address of the RPC server used for inter node communications.
    pub fn rpc_server_addr(&self) -> SocketAddr {
        self.server_addr
    }

    /// Returns the metrics of the service.
    pub fn metrics(&self) -> &ServiceMetrics {
        &self.metrics
    }

    /// Returns the identifiers of the nodes registered in the service.
    pub fn local_nodes(&self) -> Vec<LocalNodeId> {
        self.local_nodes.load().keys().cloned().collect()
    }

    pub(crate) fn generate_node_id(&self) -> NodeId {
        let local_id = LocalNodeId::new(self.next_local_id.fetch_add(1, Ordering::SeqCst) as u64);
        NodeId::new(self.server_addr, local_id)
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

            self.metrics.destination_unknown_messages.increment();
            let missing = NodeId::new(self.server_addr, id.clone());
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
