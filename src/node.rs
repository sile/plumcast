use fibers::sync::mpsc;
use futures::{Async, Poll, Stream};
use hyparview::{Action as HyparviewAction, Node as HyparviewNode};
//use plumtree::Node as PlumtreeNode;
use std::net::SocketAddr;

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
    name: NodeName,
    service: ServiceHandle,
    message_rx: mpsc::Receiver<Message>,
    hyparview_node: HyparviewNode<NodeId>,
    //    plumtree_node: PlumtreeNode,
}
impl Node {
    pub fn new(name: NodeName, service: ServiceHandle) -> Node {
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
        Node {
            name,
            service,
            message_rx,
            hyparview_node: HyparviewNode::new(id),
        }
    }
    pub fn join(&mut self, contact_peer: NodeId) {
        self.hyparview_node.join(contact_peer);
    }
    pub fn broadcast(&mut self, message: Message) {}
    pub fn leave(&mut self) {
        // TODO: self.hyparview_node.leave();
    }

    fn handle_hyparview_action(&mut self, action: HyparviewAction<NodeId>) {
        match action {
            HyparviewAction::Send { .. } => unimplemented!(),
            HyparviewAction::Notify { .. } => unimplemented!(),
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
