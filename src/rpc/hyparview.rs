use fibers_rpc::{Cast, ProcedureId};
use hyparview::message::{
    ForwardJoinMessage, JoinMessage, NeighborMessage, ShuffleMessage, ShuffleReplyMessage,
};

use codec::hyparview::{
    ForwardJoinMessageDecoder, ForwardJoinMessageEncoder, JoinMessageDecoder, JoinMessageEncoder,
    NeighborMessageDecoder, NeighborMessageEncoder, ShuffleMessageDecoder, ShuffleMessageEncoder,
    ShuffleReplyMessageDecoder, ShuffleReplyMessageEncoder,
};
use {LocalNodeId, NodeId};

#[derive(Debug)]
pub struct JoinCast;
impl Cast for JoinCast {
    const ID: ProcedureId = ProcedureId(0x17CC_0000);
    const NAME: &'static str = "hyparview.join";

    type Notification = (LocalNodeId, JoinMessage<NodeId>);
    type Decoder = JoinMessageDecoder;
    type Encoder = JoinMessageEncoder;
}

#[derive(Debug)]
pub struct ForwardJoinCast;
impl Cast for ForwardJoinCast {
    const ID: ProcedureId = ProcedureId(0x17CC_0001);
    const NAME: &'static str = "hyparview.forward_join";

    type Notification = (LocalNodeId, ForwardJoinMessage<NodeId>);
    type Decoder = ForwardJoinMessageDecoder;
    type Encoder = ForwardJoinMessageEncoder;
}

#[derive(Debug)]
pub struct NeighborCast;
impl Cast for NeighborCast {
    const ID: ProcedureId = ProcedureId(0x17CC_0002);
    const NAME: &'static str = "hyparview.neighbor";

    type Notification = (LocalNodeId, NeighborMessage<NodeId>);
    type Decoder = NeighborMessageDecoder;
    type Encoder = NeighborMessageEncoder;
}

#[derive(Debug)]
pub struct ShuffleCast;
impl Cast for ShuffleCast {
    const ID: ProcedureId = ProcedureId(0x17CC_0003);
    const NAME: &'static str = "hyparview.shuffle";

    type Notification = (LocalNodeId, ShuffleMessage<NodeId>);
    type Decoder = ShuffleMessageDecoder;
    type Encoder = ShuffleMessageEncoder;
}

#[derive(Debug)]
pub struct ShuffleReplyCast;
impl Cast for ShuffleReplyCast {
    const ID: ProcedureId = ProcedureId(0x17CC_0004);
    const NAME: &'static str = "hyparview.shuffle_reply";

    type Notification = (LocalNodeId, ShuffleReplyMessage<NodeId>);
    type Decoder = ShuffleReplyMessageDecoder;
    type Encoder = ShuffleReplyMessageEncoder;
}
