use hyparview::message::ProtocolMessage as HyparviewMessage;
use plumtree::message::ProtocolMessage as PlumtreeMessage;

use node::{MessagePayload, System};
use NodeId;

pub mod hyparview;
pub mod plumtree;

#[derive(Debug)]
pub enum RpcMessage<M: MessagePayload> {
    Hyparview(HyparviewMessage<NodeId>),
    Plumtree(PlumtreeMessage<System<M>>),
}
