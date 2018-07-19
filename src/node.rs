use fibers::sync::mpsc;
use futures::{Async, Poll, Stream};
use plumtree::message::Message as PlumtreeAppMessage;
use slog::{Discard, Logger};
use std::collections::VecDeque;
use std::fmt;
use std::time::Duration;

use hyparview_misc::{new_hyparview_node, HyparviewAction, HyparviewNode};
use plumtree_misc::{PlumtreeAction, PlumtreeNode};
use rpc::RpcMessage;
use {
    Clock, Error, ErrorKind, LocalNodeId, Message, MessageId, MessagePayload, NodeId, ServiceHandle,
};

const HYPARVIEW_SHUFFLE_INTERVAL_TICKS: u64 = 59;
const HYPARVIEW_SYNC_INTERVAL_TICKS: u64 = 31;
const HYPARVIEW_FILL_INTERVAL_TICKS: u64 = 20;

/// The builder of [`Node`].
///
/// [`Node`]: ./struct.Node.html
pub struct NodeBuilder {
    logger: Logger,
    tick_interval: Duration,
}
impl NodeBuilder {
    /// Makes a new `NodeBuilder` instance with the default settings.
    pub fn new() -> Self {
        NodeBuilder {
            logger: Logger::root(Discard, o!()),
            tick_interval: Duration::from_millis(100),
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
    /// The default value is `Duration::from_millis(100)`.
    pub fn tick_interval(&mut self, interval: Duration) -> &mut Self {
        self.tick_interval = interval;
        self
    }

    /// Builds a [`Node`] instance with the specified settings.
    ///
    /// [`Node`]: ./struct.Node.html
    pub fn finish<M: MessagePayload>(&self, service: ServiceHandle<M>) -> Node<M> {
        let id = service.generate_node_id();
        let logger = self.logger.new(o!{"node_id" => id.to_string()});
        let (message_tx, message_rx) = mpsc::channel();
        let handle = NodeHandle {
            local_id: id.local_id(),
            message_tx,
        };
        service.register_local_node(handle);
        Node {
            logger,
            service,
            message_rx,
            hyparview_node: new_hyparview_node(id.clone()),
            plumtree_node: PlumtreeNode::new(id.clone()),
            message_seqno: 0,
            deliverable_messages: VecDeque::new(),
            clock: Clock::new(self.tick_interval),
        }
    }
}
impl Default for NodeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Node<M: MessagePayload> {
    logger: Logger,
    service: ServiceHandle<M>,
    message_rx: mpsc::Receiver<RpcMessage<M>>,
    hyparview_node: HyparviewNode,
    plumtree_node: PlumtreeNode<M>,
    message_seqno: u64,
    deliverable_messages: VecDeque<Message<M>>,
    clock: Clock,
}
impl<M: MessagePayload> fmt::Debug for Node<M> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO:
        write!(f, "Node {{ .. }}")
    }
}
impl<M: MessagePayload> Node<M> {
    pub fn new(service: ServiceHandle<M>) -> Self {
        NodeBuilder::new().finish(service)
    }

    pub fn id(&self) -> NodeId {
        *self.plumtree_node().id()
    }

    pub fn clock(&self) -> &Clock {
        &self.clock
    }

    pub fn join(&mut self, contact_peer: NodeId) {
        info!(
            self.logger,
            "Joins a group by contacting to {:?}", contact_peer
        );
        self.hyparview_node.join(contact_peer);
    }

    pub fn broadcast(&mut self, message: M) {
        let mid = MessageId::new(self.id(), self.message_seqno);
        self.message_seqno += 1;
        debug!(self.logger, "Starts broadcasting a message: {:?}", mid);

        let m = PlumtreeAppMessage {
            id: mid,
            payload: message,
        };
        self.plumtree_node.broadcast_message(m);
    }

    pub fn forget_message(&mut self, message_id: &MessageId) {
        self.plumtree_node.forget_message(message_id);
    }

    pub fn hyparview_node(&self) -> &HyparviewNode {
        &self.hyparview_node
    }

    pub fn plumtree_node(&self) -> &PlumtreeNode<M> {
        &self.plumtree_node
    }

    fn handle_hyparview_action(&mut self, action: HyparviewAction) {
        use hyparview::Action;

        match action {
            Action::Send {
                destination,
                message,
            } => {
                warn!(self.logger, "[TODO] Send: {:?}", message);
                let message = RpcMessage::Hyparview(message);
                if let Err(e) = self.service.send_message(destination.clone(), message) {
                    // TODO: metrics
                    warn!(
                        self.logger,
                        "Cannot send HyParView message to {:?}: {}", destination, e
                    );
                    //if self.hyparview_node.active_view().contains(&destination) {
                    self.hyparview_node.disconnect(&destination);
                    //}
                }
            }
            Action::Notify { event } => {
                use hyparview::Event;
                match event {
                    Event::NeighborDown { node } => {
                        info!(self.logger, "Neighbor down: {:?}", node);
                        self.plumtree_node.handle_neighbor_down(&node);
                    }
                    Event::NeighborUp { node } => {
                        info!(self.logger, "Neighbor up: {:?}", node);
                        self.plumtree_node.handle_neighbor_up(&node);
                    }
                }
            }
            Action::Disconnect { node } => {
                info!(self.logger, "Disconnected: {:?}", node);
            }
        }
    }

    fn handle_plumtree_action(&mut self, action: PlumtreeAction<M>) {
        use plumtree::Action;

        match action {
            Action::Send {
                destination,
                message,
            } => {
                debug!(
                    self.logger,
                    "[ACTION] Sends a plumtree message: destination={:?}", destination,
                );
                let message = RpcMessage::Plumtree(message);
                if let Err(e) = self.service.send_message(destination.clone(), message) {
                    // TODO: metrics
                    warn!(
                        self.logger,
                        "Cannot send Plumtree message to {:?}: {}", destination, e
                    );
                    self.hyparview_node.disconnect(&destination);
                }
            }
            Action::Deliver { message } => {
                debug!(
                    self.logger,
                    "[ACTION] Delivers an application message: {:?}", message.id
                );
                // TODO: metrics
                self.deliverable_messages.push_back(Message::new(message));
            }
        }
    }

    fn handle_rpc_message(&mut self, message: RpcMessage<M>) {
        match message {
            RpcMessage::Hyparview(m) => {
                warn!(self.logger, "[TODO] Recv: {:?}", m); // TODO: remove
                self.hyparview_node.handle_protocol_message(m);
            }
            RpcMessage::Plumtree(m) => {
                debug!(self.logger, "Received a plumtree message");
                self.plumtree_node.handle_protocol_message(m);
            }
        }
    }

    fn leave(&self) {
        use hyparview::message::{DisconnectMessage, ProtocolMessage};

        for peer in self.hyparview_node.active_view().iter().cloned() {
            let message = DisconnectMessage { sender: self.id() };
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
        while track!(self.clock.poll())?.is_ready() {
            self.plumtree_node.tick();
            if self.clock.ticks() % HYPARVIEW_SHUFFLE_INTERVAL_TICKS == 0 {
                self.hyparview_node.shuffle_passive_view();
            }
            if self.clock.ticks() % HYPARVIEW_FILL_INTERVAL_TICKS == 0
                || self.hyparview_node.active_view().is_empty()
            {
                self.hyparview_node.fill_active_view();
            }
            if self.clock.ticks() % HYPARVIEW_SYNC_INTERVAL_TICKS == 0 {
                self.hyparview_node.sync_active_view();
            }
        }

        let mut did_something = true;
        while did_something {
            did_something = false;

            if let Some(message) = self.deliverable_messages.pop_front() {
                return Ok(Async::Ready(Some(message)));
            }

            while let Some(action) = self.hyparview_node.poll_action() {
                self.handle_hyparview_action(action);
                did_something = true;
            }
            while let Some(action) = self.plumtree_node.poll_action() {
                self.handle_plumtree_action(action);
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
        self.leave();
    }
}

#[derive(Clone)]
pub struct NodeHandle<M: MessagePayload> {
    local_id: LocalNodeId,
    message_tx: mpsc::Sender<RpcMessage<M>>,
}
impl<M: MessagePayload> fmt::Debug for NodeHandle<M> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "NodeHandle {{ local_id: {:?}, .. }}", self.local_id)
    }
}
impl<M: MessagePayload> NodeHandle<M> {
    pub fn local_id(&self) -> LocalNodeId {
        self.local_id
    }

    pub fn send_rpc_message(&self, message: RpcMessage<M>) {
        let _ = self.message_tx.send(message);
    }
}
