use hyparview::message::ProtocolMessage as HyparviewMessage;

use NodeId;

pub mod hyparview;

#[derive(Debug)]
pub enum RpcMessage {
    Hyparview(HyparviewMessage<NodeId>),
}
