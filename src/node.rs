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
pub struct Message;

// TODO: control (or internal) message

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeId {
    pub addr: SocketAddr,
    pub name: NodeName,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeName(String);
impl NodeName {
    pub fn new<T: Into<String>>(name: T) -> Self {
        NodeName(name.into())
    }
}

#[derive(Debug, Clone)]
pub struct NodeHandle {
    name: NodeName,
    message_tx: mpsc::Sender<Message>,
}
impl NodeHandle {
    pub fn name(&self) -> &NodeName {
        &self.name
    }
}

#[derive(Debug)]
pub struct Node {
    logger: Logger,
    name: NodeName,
    service: ServiceHandle,
    message_rx: mpsc::Receiver<Message>,
    hyparview_node: HyparviewNode<NodeId, StdRng>,
    //    plumtree_node: PlumtreeNode,
}
impl Node {
    pub fn new(logger: Logger, name: NodeName, service: ServiceHandle) -> Node {
        let id = NodeId {
            name: name.clone(),
            addr: service.server_addr,
        };
        let (message_tx, message_rx) = mpsc::channel();
        let handle = NodeHandle {
            name: name.clone(),
            message_tx,
        };
        service.register_local_node(handle);
        let rng = StdRng::from_seed(rand::thread_rng().gen());
        Node {
            logger,
            name,
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
        // TODO: self.hyparview_node.leave();
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
        self.service.deregister_local_node(self.name.clone());
    }
}
