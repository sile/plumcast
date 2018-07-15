use fibers::sync::mpsc;
use futures::{Async, Poll, Stream};
use hyparview::{self, Action as HyparviewAction, Node as HyparviewNode};
//use plumtree::Node as PlumtreeNode;
use rand::{self, Rng, SeedableRng, StdRng};
use slog::Logger;
use std::net::SocketAddr;

use rpc::RpcMessage;
use Error;
use ServiceHandle;

// application message
#[derive(Debug)]
pub struct Message; // TODO: delete

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
    message_tx: mpsc::Sender<Message>,
}
impl NodeHandle {
    pub fn local_id(&self) -> LocalNodeId {
        self.local_id
    }
}

#[derive(Debug)]
pub struct Node {
    logger: Logger,
    local_id: LocalNodeId,
    service: ServiceHandle,
    message_rx: mpsc::Receiver<Message>,
    hyparview_node: HyparviewNode<NodeId, StdRng>,
    //    plumtree_node: PlumtreeNode,
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

    pub fn leave(&mut self) {
        self.hyparview_node.leave();
    }

    fn handle_hyparview_action(&mut self, action: HyparviewAction<NodeId>) {
        match action {
            HyparviewAction::Send {
                destination,
                message,
            } => {
                let message = RpcMessage::Hyparview(message);
                self.service.send_message(destination, message)
            }
            HyparviewAction::Notify { event } => unimplemented!("{:?}", event),
            HyparviewAction::Disconnect { .. } => unimplemented!(),
        }
    }
}
impl Stream for Node {
    type Item = Message;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        while let Some(action) = self.hyparview_node.poll_action() {
            self.handle_hyparview_action(action);
        }
        Ok(Async::NotReady)
    }
}
impl Drop for Node {
    fn drop(&mut self) {
        self.service.deregister_local_node(self.local_id);
    }
}
