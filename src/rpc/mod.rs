use message::MessagePayload;
use misc::{HyparviewMessage, PlumtreeMessage};

pub mod hyparview;
pub mod plumtree;

#[derive(Debug)]
pub enum RpcMessage<M: MessagePayload> {
    Hyparview(HyparviewMessage),
    Plumtree(PlumtreeMessage<M>),
}
