use fibers::sync::mpsc;
use futures::{Async, Poll, Stream};
use hyparview::{self, Action as HyparviewAction, Node as HyparviewNode};
//use plumtree::Node as PlumtreeNode;
use rand::{self, Rng, SeedableRng, StdRng};
use slog::Logger;
use std::net::SocketAddr;

use rpc::RpcMessage;
use ServiceHandle;
use {Error, ErrorKind};

// application message
#[derive(Debug)]
pub struct Message;

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
pub struct NodeHandle {
    local_id: LocalNodeId,
    message_tx: mpsc::Sender<RpcMessage>,
}
impl NodeHandle {
    pub fn local_id(&self) -> LocalNodeId {
        self.local_id
    }

    pub fn send_rpc_message(&self, message: RpcMessage) {
        let _ = self.message_tx.send(message);
    }
}

#[derive(Debug)]
pub struct Node {
    logger: Logger,
    id: NodeId,
    local_id: LocalNodeId, // TODO: remove
    service: ServiceHandle,
    message_rx: mpsc::Receiver<RpcMessage>,
    hyparview_node: HyparviewNode<NodeId, StdRng>,
}
impl Node {
    pub fn new(logger: Logger, service: ServiceHandle) -> Node {
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
                id,
                hyparview::NodeOptions::new().set_rng(rng),
            ),
        }
    }

    pub fn join(&mut self, contact_peer: NodeId) {
        info!(
            self.logger,
            "Joins a group by contacting to {:?}", contact_peer
        );
        self.hyparview_node.join(contact_peer);
    }

    pub fn broadcast(&mut self, message: Message) {}

    fn handle_hyparview_action(&mut self, action: HyparviewAction<NodeId>) {
        match action {
            HyparviewAction::Send {
                destination,
                message,
            } => {
                warn!(self.logger, "[TODO] Send: {:?}", message);
                let message = RpcMessage::Hyparview(message);
                // TODO: handle error (i.e., disconnection)
                self.service.send_message(destination, message);
            }
            HyparviewAction::Notify { event } => warn!(self.logger, "[TODO] Event: {:?}", event),
            HyparviewAction::Disconnect { node } => {
                info!(self.logger, "Disconnected: {:?}", node);
            }
        }
    }

    fn handle_rpc_message(&mut self, message: RpcMessage) {
        match message {
            RpcMessage::Hyparview(m) => {
                warn!(self.logger, "[TODO] Recv: {:?}", m); // TODO: remove
                self.hyparview_node.handle_protocol_message(m);
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
            self.service.send_message(peer, message);
        }
    }
}
impl Stream for Node {
    type Item = Message;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        let mut did_something = true;
        while did_something {
            did_something = false;

            while let Some(action) = self.hyparview_node.poll_action() {
                self.handle_hyparview_action(action);
                did_something = true;
            }
            while let Async::Ready(message) = self.message_rx.poll().expect("Never fails") {
                let message = track_assert_some!(message, ErrorKind::Other, "Service down");
                self.handle_rpc_message(message);
                did_something = true;
            }

            // TODO: call hyperview shuffle/fill/sync periodically
        }
        Ok(Async::NotReady)
    }
}
impl Drop for Node {
    fn drop(&mut self) {
        self.service.deregister_local_node(self.local_id);
        self.leave();
    }
}
