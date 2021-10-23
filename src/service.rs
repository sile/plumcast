//! [`Service`] and related components.
//!
//! [`Service`]: ./struct.Service.html
use crate::message::MessagePayload;
use crate::metrics::{NodeMetrics, ServiceMetrics};
use crate::misc::ArcSpawn;
use crate::node::{GenerateLocalNodeId, LocalNodeId, NodeHandle, NodeId};
use crate::node_id_generator::ArcLocalNodeIdGenerator;
use crate::rpc::{self, RpcMessage};
use crate::{Error, ErrorKind, Result};
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
use slog::{Discard, Logger};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

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

    /// Returns a mutable reference to the RPC server builder.
    pub fn rpc_server_builder_mut(&mut self) -> &mut RpcServerBuilder {
        &mut self.rpc_server_builder
    }

    /// Builds a [`Service`] with the given settings.
    ///
    /// [`Service`]: ./struct.Service.html
    pub fn finish<S, M, G>(mut self, spawner: S, local_id_gen: G) -> Service<M>
    where
        S: Spawn + Send + Sync + 'static,
        M: MessagePayload,
        G: GenerateLocalNodeId,
    {
        let spawner = ArcSpawn::new(spawner);
        let (command_tx, command_rx) = mpsc::channel();
        let rpc_client_service = self.rpc_client_service_builder.finish(spawner.clone());

        let metrics = ServiceMetrics::new(self.metrics.clone());
        let removed_nodes_metrics = NodeMetrics::new(self.metrics.clone());
        let handle = ServiceHandle {
            server_addr: self.server_addr,
            command_tx,
            rpc_service: rpc_client_service.handle(),
            local_nodes: Default::default(),
            local_id_gen: ArcLocalNodeIdGenerator::new(local_id_gen),
            metrics: metrics.clone(),
            metric_builder: Arc::new(Mutex::new(self.metrics)),
        };

        rpc::hyparview::register_handlers(&mut self.rpc_server_builder, &handle);
        rpc::plumtree::register_handlers(&mut self.rpc_server_builder, &handle);
        let rpc_server = self.rpc_server_builder.finish(spawner);

        Service {
            logger: self.logger.clone(),
            command_rx,
            rpc_server,
            rpc_client_service,
            handle,
            metrics,
            removed_nodes_metrics,
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
/// [`Node`]: ../node/struct.Node.html
#[derive(Debug)]
#[must_use = "futures do nothing unless polled"]
pub struct Service<M: MessagePayload> {
    logger: Logger,
    command_rx: mpsc::Receiver<Command<M>>, // NOTE: infinite stream
    rpc_server: RpcServer<ArcSpawn>,
    rpc_client_service: RpcClientService,
    handle: ServiceHandle<M>,
    metrics: ServiceMetrics,
    removed_nodes_metrics: NodeMetrics,
}
impl<M> Service<M>
where
    M: MessagePayload,
{
    /// Makes a new `Service` instance with the default settings.
    ///
    /// If you want to customize settings, please use [`ServiceBuilder`] instead.
    ///
    /// [`ServiceBuilder`]: ./struct.ServiceBuilder.html
    pub fn new<S, G>(rpc_server_bind_addr: SocketAddr, spawner: S, local_id_gen: G) -> Self
    where
        S: Spawn + Send + Sync + 'static,
        G: GenerateLocalNodeId,
    {
        ServiceBuilder::new(rpc_server_bind_addr).finish(spawner, local_id_gen)
    }

    /// Returns the handle of the service.
    pub fn handle(&self) -> ServiceHandle<M> {
        self.handle.clone()
    }

    /// Returns a reference to the RPC server of the service.
    pub fn rpc_server(&self) -> &RpcServer<ArcSpawn> {
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
                    if let Some(n) = nodes.remove(&node) {
                        self.removed_nodes_metrics.add(n.metrics());
                    }
                    nodes
                });
            }
        }
        Ok(())
    }
}
impl<M> Future for Service<M>
where
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
impl<M: MessagePayload> Drop for Service<M> {
    fn drop(&mut self) {
        let old = self.handle.local_nodes.swap(HashMap::new());
        self.metrics.deregistered_nodes.add_u64(old.len() as u64);
        for node in old.values() {
            self.removed_nodes_metrics.add(node.metrics());
        }
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
    local_id_gen: ArcLocalNodeIdGenerator,
    metrics: ServiceMetrics,
    metric_builder: Arc<Mutex<MetricBuilder>>,
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

    pub(crate) fn metric_builder(&self) -> MetricBuilder {
        if let Ok(m) = self.metric_builder.lock() {
            m.clone()
        } else {
            MetricBuilder::new()
        }
    }

    pub(crate) fn generate_node_id(&self) -> NodeId {
        let local_id = self.local_id_gen.generate_local_node_id();
        NodeId::new(self.server_addr, local_id)
    }

    pub(crate) fn get_local_node(&self, local_id: LocalNodeId) -> Option<NodeHandle<M>> {
        self.local_nodes.load().get(&local_id).cloned()
    }

    pub(crate) fn get_local_node_or_disconnect(
        &self,
        id: LocalNodeId,
        sender: &NodeId,
    ) -> Option<NodeHandle<M>> {
        if let Some(node) = self.local_nodes.load().get(&id).cloned() {
            Some(node)
        } else {
            use hyparview::message::{DisconnectMessage, ProtocolMessage};

            self.metrics.destination_unknown_messages.increment();
            let missing = NodeId::new(self.server_addr, id);
            let message = DisconnectMessage {
                sender: missing,
                alive: false,
            };
            let message = ProtocolMessage::Disconnect(message);
            let _ = self.send_message(*sender, RpcMessage::Hyparview(message));
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
                use crate::rpc::hyparview as hv;
                use hyparview::message::ProtocolMessage;

                match m {
                    ProtocolMessage::Join(m) => {
                        track!(hv::join_cast(peer, m, &self.rpc_service))?;
                    }
                    ProtocolMessage::ForwardJoin(m) => {
                        track!(hv::forward_join_cast(peer, m, &self.rpc_service))?;
                    }
                    ProtocolMessage::Neighbor(m) => {
                        track!(hv::neighbor_cast(peer, m, &self.rpc_service))?;
                    }
                    ProtocolMessage::Shuffle(m) => {
                        track!(hv::shuffle_cast(peer, m, &self.rpc_service))?;
                    }
                    ProtocolMessage::ShuffleReply(m) => {
                        track!(hv::shuffle_reply_cast(peer, m, &self.rpc_service))?;
                    }
                    ProtocolMessage::Disconnect(m) => {
                        track!(hv::disconnect_cast(peer, m, &self.rpc_service))?;
                    }
                }
            }
            RpcMessage::Plumtree(m) => {
                use crate::rpc::plumtree as pt;
                use plumtree::message::ProtocolMessage;

                match m {
                    ProtocolMessage::Gossip(m) => {
                        track!(pt::gossip_cast(peer, m, &self.rpc_service))?;
                    }
                    ProtocolMessage::Ihave(m) => {
                        track!(pt::ihave_cast(peer, m, &self.rpc_service))?;
                    }
                    ProtocolMessage::Graft(m) => {
                        track!(pt::graft_cast(peer, m, &self.rpc_service))?;
                    }
                    ProtocolMessage::Prune(m) => {
                        track!(pt::prune_cast(peer, m, &self.rpc_service))?;
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
