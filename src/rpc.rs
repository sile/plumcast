use fibers_rpc::{Cast, ProcedureId};
use hyparview;

use NodeId;

type HyparviewMessage = hyparview::Message<NodeId>;

#[derive(Debug)]
pub enum RpcMessage {
    Hyparview(HyparviewMessage),
}

// #[derive(Debug)]
// pub struct HyparviewJoin;
// impl Cast for HyparviewJoin {
//     const ID: ProcedureId = ProcedureId(0x0000_0000);
//     const NAME: &'static str = "hyparview.join";

//     // type Notification = hyparview::Message

//     // /// Notification message encoder.
//     // type Encoder: bytecodec::Encode<Item = Self::Notification> + Send + 'static;

//     // /// Notification message decoder.
//     // type Decoder: bytecodec::Decode<Item = Self::Notification> + Send + 'static;
// }
