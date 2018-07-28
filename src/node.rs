//! [`Node`] and related components.
//!
//! [`Node`]: ./node/struct.Node.html
use fibers::sync::mpsc;
use fibers::time::timer::{self, Timeout};
use futures::{Async, Future, Poll, Stream};
use plumtree::message::Message as PlumtreeAppMessage;
use plumtree::time::{Clock, NodeTime};
use rand::{self, Rng, SeedableRng, StdRng};
use slog::{Discard, Logger};
use std::fmt;
use std::time::Duration;

use message::{Message, MessageId, MessagePayload};
use metrics::NodeMetrics;
use misc::{
    HyparviewAction, HyparviewNode, HyparviewNodeOptions, PlumtreeAction, PlumtreeNode,
    PlumtreeNodeOptions,
};
use rpc::RpcMessage;
use service::ServiceHandle;
use {Error, ErrorKind};

pub use node_id::{LocalNodeId, NodeId};
pub use node_id_generator::{
    GenerateLocalNodeId, SerialLocalNodeIdGenerator, UnixtimeLocalNodeIdGenerator,
};

/// The builder of [`Node`].
///
/// [`Node`]: ./struct.Node.html
#[derive(Debug, Clone)]
pub struct NodeBuilder {
    logger: Logger,
    hyparview_options: HyparviewNodeOptions,
    plumtree_options: PlumtreeNodeOptions,
    params: Parameters,
}
impl NodeBuilder {
    /// Makes a new `NodeBuilder` instance with the default settings.
    pub fn new() -> Self {
        let params = Parameters {
            tick_interval: Duration::from_millis(200),
            hyparview_shuffle_interval: Duration::from_secs(300),
            hyparview_sync_active_view_interval: Duration::from_secs(60),
            hyparview_fill_active_view_interval: Duration::from_secs(30),
        };
        NodeBuilder {
            logger: Logger::root(Discard, o!()),
            hyparview_options: HyparviewNodeOptions::default(),
            plumtree_options: PlumtreeNodeOptions::default(),
            params,
        }
    }

    /// Sets the logger used by the node.
    ///
    /// The default value is `Logger::root(Discard, o!())`.
    pub fn logger(&mut self, logger: Logger) -> &mut Self {
        self.logger = logger;
        self
    }

    /// Sets the unit of the node local [`Clock`].
    ///
    /// The default value is `Duration::from_millis(200)`.
    pub fn tick_interval(&mut self, interval: Duration) -> &mut Self {
        self.params.tick_interval = interval;
        self
    }

    /// Sets the execution interval of `HyparviewNode::shuffle_passive_view()` method.
    ///
    /// The default value is `Duration::from_secs(300)`.
    pub fn hyparview_shuffle_interval(&mut self, interval: Duration) -> &mut Self {
        self.params.hyparview_shuffle_interval = interval;
        self
    }

    /// Sets the execution interval of `HyparviewNode::shuffle_passive_view()` method.
    ///
    /// The default value is `Duration::from_secs(60)`.
    pub fn hyparview_sync_active_view_interval(&mut self, interval: Duration) -> &mut Self {
        self.params.hyparview_sync_active_view_interval = interval;
        self
    }

    /// Sets the execution interval of `HyparviewNode::shuffle_passive_view()` method.
    ///
    /// The default value is `Duration::from_secs(30)`.
    pub fn hyparview_fill_active_view_interval(&mut self, interval: Duration) -> &mut Self {
        self.params.hyparview_fill_active_view_interval = interval;
        self
    }

    /// Sets the options for the underlying HyParView node.
    ///
    /// The default value is `HyparviewNodeOptions::default()`.
    pub fn hyparview_options(&mut self, options: HyparviewNodeOptions) -> &mut Self {
        self.hyparview_options = options;
        self
    }

    /// Sets the options for the underlying Plumtree node.
    ///
    /// The default value is `PlumtreeNodeOptions::default()`.
    pub fn plumtree_options(&mut self, options: PlumtreeNodeOptions) -> &mut Self {
        self.plumtree_options = options;
        self
    }

    /// Builds a [`Node`] instance with the specified settings.
    ///
    /// [`Node`]: ./struct.Node.html
    pub fn finish<M: MessagePayload>(&self, service: ServiceHandle<M>) -> Node<M> {
        let id = service.generate_node_id();
        let logger = self.logger.new(o!{"node_id" => id.to_string()});
        let metrics = NodeMetrics::new(service.metric_builder());
        let (message_tx, message_rx) = mpsc::channel();
        let handle = NodeHandle {
            local_id: id.local_id(),
            message_tx,
            metrics: metrics.clone(),
        };
        let rng = StdRng::from_seed(rand::thread_rng().gen());
        service.register_local_node(handle);

        let plumtree_node = PlumtreeNode::with_options(id, self.plumtree_options.clone());
        let now = plumtree_node.clock().now();
        let hyparview_shuffle_time = now + gen_interval(self.params.hyparview_shuffle_interval);
        let hyparview_sync_active_view_time =
            now + gen_interval(self.params.hyparview_sync_active_view_interval);
        let hyparview_fill_active_view_time =
            now + gen_interval(self.params.hyparview_fill_active_view_interval);
        Node {
            logger,
            service,
            message_rx,
            hyparview_node: HyparviewNode::with_options(id, rng, self.hyparview_options.clone()),
            plumtree_node,
            message_seqno: 0,
            hyparview_shuffle_time,
            hyparview_sync_active_view_time,
            hyparview_fill_active_view_time,
            tick_timeout: timer::timeout(self.params.tick_interval),
            params: self.params.clone(),
            metrics,
        }
    }
}
impl Default for NodeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Node that broadcasts and receives messages.
#[derive(Debug)]
#[must_use = "streams do nothing unless polled"]
pub struct Node<M: MessagePayload> {
    logger: Logger,
    service: ServiceHandle<M>,
    message_rx: mpsc::Receiver<RpcMessage<M>>,
    hyparview_node: HyparviewNode,
    plumtree_node: PlumtreeNode<M>,
    message_seqno: u64,
    hyparview_shuffle_time: NodeTime,
    hyparview_sync_active_view_time: NodeTime,
    hyparview_fill_active_view_time: NodeTime,
    tick_timeout: Timeout,
    params: Parameters,
    metrics: NodeMetrics,
}
impl<M: MessagePayload> Node<M> {
    /// Makes a new `Node` instance with the default settings.
    ///
    /// If you want to customize settings, please use [`NodeBuilder`] instead.
    ///
    /// [`NodeBuilder`]: ./struct.NodeBuilder.html
    pub fn new(service: ServiceHandle<M>) -> Self {
        NodeBuilder::new().finish(service)
    }

    /// Returns the identifier of the node.
    pub fn id(&self) -> NodeId {
        *self.plumtree_node().id()
    }

    /// Joins the cluster to which the given contact node belongs.
    pub fn join(&mut self, contact_node: NodeId) {
        info!(
            self.logger,
            "Joins a cluster by contacting to {:?}", contact_node
        );
        self.hyparview_node.join(contact_node);
    }

    /// Broadcasts a message.
    ///
    /// Note that the message will also be delivered to the sender node.
    pub fn broadcast(&mut self, message_payload: M) {
        let id = MessageId::new(self.id(), self.message_seqno);
        self.message_seqno += 1;
        debug!(self.logger, "Starts broadcasting a message: {:?}", id);

        let m = PlumtreeAppMessage {
            id,
            payload: message_payload,
        };
        self.plumtree_node.broadcast_message(m);
        self.metrics.broadcasted_messages.increment()
    }

    /// Forgets the specified message.
    ///
    /// For preventing memory shortage, this method needs to be called appropriately.
    pub fn forget_message(&mut self, message_id: &MessageId) {
        if self.plumtree_node.forget_message(message_id) {
            self.metrics.forgot_messages.increment();
        } else {
            self.metrics.forget_unknown_message_errors.increment();
        }
    }

    /// Returns a reference to the underlying HyParView node.
    pub fn hyparview_node(&self) -> &HyparviewNode {
        &self.hyparview_node
    }

    /// Returns a reference to the underlying Plumtree node.
    pub fn plumtree_node(&self) -> &PlumtreeNode<M> {
        &self.plumtree_node
    }

    /// Returns the clock of the node.
    pub fn clock(&self) -> &Clock {
        self.plumtree_node.clock()
    }

    /// Returns the metrics of the service.
    pub fn metrics(&self) -> &NodeMetrics {
        &self.metrics
    }

    fn handle_hyparview_action(&mut self, action: HyparviewAction) {
        use hyparview::{Action, Event};

        match action {
            Action::Send {
                destination,
                message,
            } => {
                debug!(
                    self.logger,
                    "Sends a HyParView message to {:?}: {:?}", destination, message
                );
                let message = RpcMessage::Hyparview(message);
                if let Err(e) = self.service.send_message(destination, message) {
                    warn!(
                        self.logger,
                        "Cannot send a HyParView message to {:?}: {}", destination, e
                    );
                    self.metrics
                        .cannot_send_hyparview_message_errors
                        .increment();
                    self.hyparview_node.disconnect(&destination, false);
                }
            }
            Action::Notify { event } => match event {
                Event::NeighborUp { node } => {
                    info!(
                        self.logger,
                        "Neighbor up: {:?} (active_view={:?})",
                        node,
                        self.hyparview_node.active_view()
                    );
                    self.metrics.connected_neighbors.increment();
                    self.plumtree_node.handle_neighbor_up(&node);
                    if self.hyparview_node.active_view().len() == 1 {
                        self.metrics.deisolated_times.increment();
                    }
                }
                Event::NeighborDown { node } => {
                    info!(
                        self.logger,
                        "Neighbor down: {:?} (active_view={:?})",
                        node,
                        self.hyparview_node.active_view()
                    );
                    self.metrics.disconnected_neighbors.increment();
                    self.plumtree_node.handle_neighbor_down(&node);
                    if self.hyparview_node.active_view().is_empty() {
                        self.metrics.isolated_times.increment();
                    }
                }
            },
            Action::Disconnect { node } => {
                info!(self.logger, "Disconnected: {:?}", node);
            }
        }
    }

    fn handle_plumtree_action(&mut self, action: PlumtreeAction<M>) -> Option<Message<M>> {
        use plumtree::Action;

        match action {
            Action::Send {
                destination,
                message,
            } => {
                debug!(self.logger, "Sends a Plumtree message to {:?}", destination,);
                let message = RpcMessage::Plumtree(message);
                if let Err(e) = self.service.send_message(destination, message) {
                    warn!(
                        self.logger,
                        "Cannot send a Plumtree message to {:?}: {}", destination, e
                    );
                    self.metrics.cannot_send_plumtree_message_errors.increment();
                    self.hyparview_node.disconnect(&destination, false);
                }
                None
            }
            Action::Deliver { message } => {
                debug!(
                    self.logger,
                    "Delivers an application message: {:?}", message.id
                );
                self.metrics.delivered_messages.increment();
                Some(Message::new(message))
            }
        }
    }

    fn handle_rpc_message(&mut self, message: RpcMessage<M>) {
        match message {
            RpcMessage::Hyparview(m) => {
                debug!(self.logger, "Received a HyParView message: {:?}", m);
                self.hyparview_node.handle_protocol_message(m);
            }
            RpcMessage::Plumtree(m) => {
                debug!(self.logger, "Received a Plumtree message");
                if !self.plumtree_node.handle_protocol_message(m) {
                    self.metrics.unknown_plumtree_node_errors.increment();
                }
            }
        }
    }

    fn handle_tick(&mut self) {
        self.plumtree_node
            .clock_mut()
            .tick(self.params.tick_interval);

        let now = self.plumtree_node.clock().now();
        if now >= self.hyparview_shuffle_time {
            self.hyparview_node.shuffle_passive_view();
            self.hyparview_shuffle_time =
                now + gen_interval(self.params.hyparview_shuffle_interval);
        }
        if now >= self.hyparview_sync_active_view_time {
            self.hyparview_node.sync_active_view();
            self.hyparview_sync_active_view_time =
                now + gen_interval(self.params.hyparview_sync_active_view_interval);
        }
        if now >= self.hyparview_fill_active_view_time {
            self.hyparview_node.fill_active_view();
            self.hyparview_fill_active_view_time =
                now + gen_interval(self.params.hyparview_fill_active_view_interval);
        }
    }

    fn leave(&self) {
        use hyparview::message::{DisconnectMessage, ProtocolMessage};

        info!(
            self.logger,
            "Leaves the current cluster: active_view={:?}",
            self.hyparview_node.active_view()
        );
        for peer in self.hyparview_node.active_view().iter().cloned() {
            let message = DisconnectMessage {
                sender: self.id(),
                alive: false,
            };
            let message = ProtocolMessage::Disconnect(message);
            let message = RpcMessage::Hyparview(message);
            let _ = self.service.send_message(peer, message);
        }
    }
}
impl<M: MessagePayload> Stream for Node<M> {
    type Item = Message<M>;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        while track!(self.tick_timeout.poll().map_err(Error::from))?.is_ready() {
            self.handle_tick();
            self.tick_timeout = timer::timeout(self.params.tick_interval);
        }

        let mut did_something = true;
        while did_something {
            did_something = false;

            while let Some(action) = self.hyparview_node.poll_action() {
                self.handle_hyparview_action(action);
                did_something = true;
            }
            while let Some(action) = self.plumtree_node.poll_action() {
                if let Some(message) = self.handle_plumtree_action(action) {
                    return Ok(Async::Ready(Some(message)));
                }
                did_something = true;
            }
            while let Async::Ready(message) = self.message_rx.poll().expect("Never fails") {
                let message = track_assert_some!(message, ErrorKind::Other, "Service down");
                self.handle_rpc_message(message);
                did_something = true;
            }
        }
        Ok(Async::NotReady)
    }
}
impl<M: MessagePayload> Drop for Node<M> {
    fn drop(&mut self) {
        self.service.deregister_local_node(self.id().local_id());

        let messages = self.metrics.delivered_messages() - self.metrics.forgot_messages();
        self.metrics.forgot_messages.add_u64(messages);

        self.leave();
    }
}

#[derive(Clone)]
pub(crate) struct NodeHandle<M: MessagePayload> {
    local_id: LocalNodeId,
    message_tx: mpsc::Sender<RpcMessage<M>>,
    metrics: NodeMetrics,
}
impl<M: MessagePayload> fmt::Debug for NodeHandle<M> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "NodeHandle {{ local_id: {:?}, .. }}", self.local_id)
    }
}
impl<M: MessagePayload> NodeHandle<M> {
    pub(crate) fn local_id(&self) -> LocalNodeId {
        self.local_id
    }

    pub(crate) fn send_rpc_message(&self, message: RpcMessage<M>) {
        let _ = self.message_tx.send(message);
    }

    pub(crate) fn metrics(&self) -> &NodeMetrics {
        &self.metrics
    }
}

#[derive(Debug, Clone)]
struct Parameters {
    tick_interval: Duration,
    hyparview_shuffle_interval: Duration,
    hyparview_sync_active_view_interval: Duration,
    hyparview_fill_active_view_interval: Duration,
}

fn gen_interval(base: Duration) -> Duration {
    let millis = base.as_secs() * 1000 + u64::from(base.subsec_millis());
    let jitter = rand::random::<u64>() % (millis / 10);
    base + Duration::from_millis(jitter)
}
