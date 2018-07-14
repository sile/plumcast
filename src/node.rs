use fibers::sync::mpsc;
use futures::Poll;
use std::net::SocketAddr;

use ServiceHandle;

#[derive(Debug)]
pub struct Message;

#[derive(Debug)]
pub struct NodeId {
    pub addr: SocketAddr,
    pub name: String,
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
}
impl Node {
    pub fn new(name: NodeName, service: ServiceHandle) -> Node {
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
        }
    }
    pub fn join(&mut self, _contact_peer: NodeId) {}
    pub fn broadcast(&mut self, (): ()) {}
    pub fn leave(&mut self) {}
    pub fn poll_message(&mut self) -> Poll<(), ()> {
        panic!()
    }
}
impl Drop for Node {
    fn drop(&mut self) {
        self.service.deregister_local_node(self.name.clone());
    }
}
