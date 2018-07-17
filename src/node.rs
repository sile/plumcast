use fibers::sync::mpsc;
use fibers::time::timer::{self, Timeout};
use futures::{Async, Future, Poll, Stream};
use hyparview::{self, Action as HyparviewAction, Node as HyparviewNode};
use plumtree::message::Message;
use plumtree::{self, Action as PlumtreeAction, Node as PlumtreeNode};
use rand::{self, Rng, SeedableRng, StdRng};
use slog::Logger;
use std::collections::VecDeque;
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::time::Duration;

use rpc::RpcMessage;
use ServiceHandle;
use {Error, ErrorKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LocalNodeId(u64);
impl LocalNodeId {
    pub fn new(id: u64) -> Self {
        LocalNodeId(id)
    }

    pub fn value(&self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeId {
    pub addr: SocketAddr, // TODO: location(?)
    pub local_id: LocalNodeId,
}

#[derive(Debug, Clone)]
pub struct NodeHandle<M: MessagePayload> {
    local_id: LocalNodeId,
    message_tx: mpsc::Sender<RpcMessage<M>>,
}
impl<M: MessagePayload> NodeHandle<M> {
    pub fn local_id(&self) -> LocalNodeId {
        self.local_id
    }

    pub fn send_rpc_message(&self, message: RpcMessage<M>) {
        let _ = self.message_tx.send(message);
    }
}
impl MessagePayload for Vec<u8> {
    type Encoder = ::bytecodec::bytes::BytesEncoder<Vec<u8>>;
    type Decoder = ::bytecodec::bytes::RemainingBytesDecoder;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MessageId {
    pub node_id: NodeId,
    pub seqno: u64,
}

#[derive(Debug)]
pub struct System<M>(PhantomData<M>);
impl<M: MessagePayload> plumtree::System for System<M> {
    type NodeId = NodeId;
    type MessageId = MessageId;
    type MessagePayload = M;
}

// TODO: remove Sync, Debug
pub trait MessagePayload: Sized + Clone + Send + Sync + 'static + ::std::fmt::Debug {
    type Encoder: ::bytecodec::Encode<Item = Self> + Default + Send + 'static + ::std::fmt::Debug; // TODO: remove Debug
    type Decoder: ::bytecodec::Decode<Item = Self> + Default + Send + 'static + ::std::fmt::Debug;
}

const TICK_MS: u64 = 1000; // TODO
const HYPARVIEW_SHUFFLE_INTERVAL_TICKS: usize = 59;
const HYPARVIEW_SYNC_INTERVAL_TICKS: usize = 31;
const HYPARVIEW_FILL_INTERVAL_TICKS: usize = 20;

#[derive(Debug)]
pub struct Node<M: MessagePayload> {
    logger: Logger,
    id: NodeId,
    local_id: LocalNodeId, // TODO: remove
    service: ServiceHandle<M>,
    message_rx: mpsc::Receiver<RpcMessage<M>>,
    hyparview_node: HyparviewNode<NodeId, StdRng>,
    plumtree_node: PlumtreeNode<System<M>>,
    message_seqno: u64,
    deliverable_messages: VecDeque<Message<System<M>>>,
    tick: Timeout, // TODO: tick_timeout
    ticks: usize,  // or clock
}
impl<M: MessagePayload> Node<M> {
    pub fn new(logger: Logger, service: ServiceHandle<M>) -> Self {
        let id = service.generate_node_id();
        let (message_tx, message_rx) = mpsc::channel();
        let handle = NodeHandle {
            local_id: id.local_id,
            message_tx,
        };
        service.register_local_node(handle);
        let rng = StdRng::from_seed(rand::thread_rng().gen());
        Node {
            logger,
            id: id.clone(),
            local_id: id.local_id,
            service,
            message_rx,
            hyparview_node: HyparviewNode::with_options(
                id.clone(),
                hyparview::NodeOptions::new().set_rng(rng),
            ),
            plumtree_node: PlumtreeNode::new(id.clone()),
            message_seqno: 0, // TODO: random (or make initial node id random)
            deliverable_messages: VecDeque::new(),
            tick: timer::timeout(Duration::from_millis(TICK_MS)),
            ticks: 0,
        }
    }

    pub fn join(&mut self, contact_peer: NodeId) {
        info!(
            self.logger,
            "Joins a group by contacting to {:?}", contact_peer
        );
        self.hyparview_node.join(contact_peer);
    }

    pub fn broadcast(&mut self, message: M) {
        warn!(self.logger, "[TODO] Broadcast: {:?}", message);
        let mid = MessageId {
            node_id: self.id.clone(),
            seqno: self.message_seqno,
        };
        self.message_seqno += 1;

        let m = Message {
            id: mid,
            payload: message,
        };
        self.plumtree_node.broadcast_message(m);
    }

    pub fn forget_message(&mut self, message_id: &MessageId) {
        self.plumtree_node.forget_message(message_id);
    }

    fn handle_hyparview_action(&mut self, action: HyparviewAction<NodeId>) {
        match action {
            HyparviewAction::Send {
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
            HyparviewAction::Notify { event } => {
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
            HyparviewAction::Disconnect { node } => {
                info!(self.logger, "Disconnected: {:?}", node);
            }
        }
    }

    fn handle_plumtree_action(&mut self, action: PlumtreeAction<System<M>>) {
        warn!(self.logger, "[TODO] Action: {:?}", action);
        match action {
            PlumtreeAction::Send {
                destination,
                message,
            } => {
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
            PlumtreeAction::Deliver { message } => {
                self.deliverable_messages.push_back(message);
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
                warn!(self.logger, "[TODO] Recv: {:?}", m); // TODO: remove
                self.plumtree_node.handle_protocol_message(m);
            }
        }
    }

    fn leave(&self) {
        for peer in self.hyparview_node.active_view().iter().cloned() {
            let message = hyparview::message::DisconnectMessage {
                sender: self.id.clone(),
            };
            let message = hyparview::message::ProtocolMessage::Disconnect(message);
            let message = RpcMessage::Hyparview(message);
            let _ = self.service.send_message(peer, message);
        }
    }
}
impl<M: MessagePayload> Stream for Node<M> {
    type Item = Message<System<M>>;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        while track!(self.tick.poll().map_err(Error::from))?.is_ready() {
            self.plumtree_node.tick();
            self.ticks += 1;
            if self.ticks % HYPARVIEW_SHUFFLE_INTERVAL_TICKS == 0 {
                self.hyparview_node.shuffle_passive_view();
            }
            if self.ticks % HYPARVIEW_FILL_INTERVAL_TICKS == 0
                || self.hyparview_node.active_view().is_empty()
            {
                self.hyparview_node.fill_active_view();
            }
            if self.ticks % HYPARVIEW_SYNC_INTERVAL_TICKS == 0 {
                self.hyparview_node.sync_active_view();
            }
            self.tick = timer::timeout(Duration::from_millis(TICK_MS));
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
        self.service.deregister_local_node(self.local_id);
        self.leave();
    }
}
