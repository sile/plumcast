use hyparview::message::ProtocolMessage as HyparviewMessage;
use plumtree::message::ProtocolMessage as PlumtreeMessage;

use node::System;
use NodeId;

pub mod hyparview;
pub mod plumtree;

#[derive(Debug)]
pub enum RpcMessage {
    Hyparview(HyparviewMessage<NodeId>),
    Plumtree(PlumtreeMessage<System>),
}
