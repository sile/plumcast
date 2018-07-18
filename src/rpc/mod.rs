use hyparview::message::ProtocolMessage as HyparviewMessage;
use plumtree::message::ProtocolMessage as PlumtreeMessage;

use node::System;
use {MessagePayload, NodeId};

pub mod hyparview;
pub mod plumtree;

#[derive(Debug)]
pub enum RpcMessage<M: MessagePayload> {
    Hyparview(HyparviewMessage<NodeId>),
    Plumtree(PlumtreeMessage<System<M>>),
}
