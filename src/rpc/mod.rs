use hyparview_misc::HyparviewMessage;
use plumtree_misc::PlumtreeMessage;
use MessagePayload;

pub mod hyparview;
pub mod plumtree;

#[derive(Debug)]
pub enum RpcMessage<M: MessagePayload> {
    Hyparview(HyparviewMessage),
    Plumtree(PlumtreeMessage<M>),
}
