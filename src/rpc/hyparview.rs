use fibers_rpc::client::ClientServiceHandle;
use fibers_rpc::server::{HandleCast, NoReply, ServerBuilder};
use fibers_rpc::{Cast, ProcedureId};
use hyparview::message::{
    DisconnectMessage, ForwardJoinMessage, JoinMessage, NeighborMessage, ProtocolMessage,
    ShuffleMessage, ShuffleReplyMessage,
};

use super::RpcMessage;
use codec::hyparview::{
    DisconnectMessageDecoder, DisconnectMessageEncoder, ForwardJoinMessageDecoder,
    ForwardJoinMessageEncoder, JoinMessageDecoder, JoinMessageEncoder, NeighborMessageDecoder,
    NeighborMessageEncoder, ShuffleMessageDecoder, ShuffleMessageEncoder,
    ShuffleReplyMessageDecoder, ShuffleReplyMessageEncoder,
};
use service::ServiceHandle;
use {LocalNodeId, NodeId};

pub fn register_handlers(rpc: &mut ServerBuilder, service: ServiceHandle) {
    rpc.add_cast_handler(JoinHandler(service.clone()));
    rpc.add_cast_handler(ForwardJoinHandler(service.clone()));
    rpc.add_cast_handler(NeighborHandler(service.clone()));
    rpc.add_cast_handler(ShuffleHandler(service.clone()));
    rpc.add_cast_handler(ShuffleReplyHandler(service.clone()));
    rpc.add_cast_handler(DisconnectHandler(service.clone()));
}

#[derive(Debug)]
pub struct JoinCast;
impl Cast for JoinCast {
    const ID: ProcedureId = ProcedureId(0x17CC_0000);
    const NAME: &'static str = "hyparview.join";

    type Notification = (LocalNodeId, JoinMessage<NodeId>);
    type Decoder = JoinMessageDecoder;
    type Encoder = JoinMessageEncoder;
}

pub fn join_cast(peer: NodeId, m: JoinMessage<NodeId>, service: &ClientServiceHandle) {
    // TODO: set options (e.g., priority, force_wakeup)
    let client = JoinCast::client(&service);
    track_try_unwrap!(client.cast(peer.addr, (peer.local_id, m))); // TODO
}

#[derive(Debug)]
struct JoinHandler(ServiceHandle);
impl HandleCast<JoinCast> for JoinHandler {
    fn handle_cast(&self, (id, m): (LocalNodeId, JoinMessage<NodeId>)) -> NoReply {
        if let Some(node) = self.0.get_local_node(&id) {
            let m = RpcMessage::Hyparview(ProtocolMessage::Join(m));
            node.send_rpc_message(m);
        } else {
            println!("# UNKNOWN NODE: {:?}, {:?}", id, m);
            // TODO: metric or log
            // TODO: reply disconnect
        }
        NoReply::done()
    }
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

pub fn forward_join_cast(
    peer: NodeId,
    m: ForwardJoinMessage<NodeId>,
    service: &ClientServiceHandle,
) {
    // TODO: set options (e.g., priority, force_wakeup)
    let client = ForwardJoinCast::client(&service);
    let _ = client.cast(peer.addr, (peer.local_id, m));
}

#[derive(Debug)]
struct ForwardJoinHandler(ServiceHandle);
impl HandleCast<ForwardJoinCast> for ForwardJoinHandler {
    fn handle_cast(&self, (id, m): (LocalNodeId, ForwardJoinMessage<NodeId>)) -> NoReply {
        if let Some(node) = self.0.get_local_node(&id) {
            let m = RpcMessage::Hyparview(ProtocolMessage::ForwardJoin(m));
            node.send_rpc_message(m);
        } else {
            println!("# UNKNOWN NODE: {:?}, {:?}", id, m);

            // TODO: metric or log
            // TODO: reply disconnect
        }
        NoReply::done()
    }
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

pub fn neighbor_cast(peer: NodeId, m: NeighborMessage<NodeId>, service: &ClientServiceHandle) {
    // TODO: set options (e.g., priority, force_wakeup)
    let client = NeighborCast::client(&service);
    let _ = client.cast(peer.addr, (peer.local_id, m));
}

#[derive(Debug)]
struct NeighborHandler(ServiceHandle);
impl HandleCast<NeighborCast> for NeighborHandler {
    fn handle_cast(&self, (id, m): (LocalNodeId, NeighborMessage<NodeId>)) -> NoReply {
        if let Some(node) = self.0.get_local_node(&id) {
            let m = RpcMessage::Hyparview(ProtocolMessage::Neighbor(m));
            node.send_rpc_message(m);
        } else {
            println!("# UNKNOWN NODE: {:?}, {:?}", id, m);

            // TODO: metric or log
            // TODO: reply disconnect
        }
        NoReply::done()
    }
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

pub fn shuffle_cast(peer: NodeId, m: ShuffleMessage<NodeId>, service: &ClientServiceHandle) {
    // TODO: set options (e.g., priority, force_wakeup)
    let client = ShuffleCast::client(&service);
    let _ = client.cast(peer.addr, (peer.local_id, m));
}

#[derive(Debug)]
struct ShuffleHandler(ServiceHandle);
impl HandleCast<ShuffleCast> for ShuffleHandler {
    fn handle_cast(&self, (id, m): (LocalNodeId, ShuffleMessage<NodeId>)) -> NoReply {
        if let Some(node) = self.0.get_local_node(&id) {
            let m = RpcMessage::Hyparview(ProtocolMessage::Shuffle(m));
            node.send_rpc_message(m);
        } else {
            println!("# UNKNOWN NODE: {:?}, {:?}", id, m);

            // TODO: metric or log
            // TODO: reply disconnect
        }
        NoReply::done()
    }
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

pub fn shuffle_reply_cast(
    peer: NodeId,
    m: ShuffleReplyMessage<NodeId>,
    service: &ClientServiceHandle,
) {
    // TODO: set options (e.g., priority, force_wakeup)
    let client = ShuffleReplyCast::client(&service);
    let _ = client.cast(peer.addr, (peer.local_id, m));
}

#[derive(Debug)]
struct ShuffleReplyHandler(ServiceHandle);
impl HandleCast<ShuffleReplyCast> for ShuffleReplyHandler {
    fn handle_cast(&self, (id, m): (LocalNodeId, ShuffleReplyMessage<NodeId>)) -> NoReply {
        if let Some(node) = self.0.get_local_node(&id) {
            let m = RpcMessage::Hyparview(ProtocolMessage::ShuffleReply(m));
            node.send_rpc_message(m);
        } else {
            println!("# UNKNOWN NODE: {:?}, {:?}", id, m);

            // TODO: metric or log
            // TODO: reply disconnect
        }
        NoReply::done()
    }
}

#[derive(Debug)]
pub struct DisconnectCast;
impl Cast for DisconnectCast {
    const ID: ProcedureId = ProcedureId(0x17CC_0005);
    const NAME: &'static str = "hyparview.disconnect";

    type Notification = (LocalNodeId, DisconnectMessage<NodeId>);
    type Decoder = DisconnectMessageDecoder;
    type Encoder = DisconnectMessageEncoder;
}

pub fn disconnect_cast(peer: NodeId, m: DisconnectMessage<NodeId>, service: &ClientServiceHandle) {
    // TODO: set options (e.g., priority, force_wakeup)
    let client = DisconnectCast::client(&service);
    track_try_unwrap!(client.cast(peer.addr, (peer.local_id, m))); // TODO
}

#[derive(Debug)]
struct DisconnectHandler(ServiceHandle);
impl HandleCast<DisconnectCast> for DisconnectHandler {
    fn handle_cast(&self, (id, m): (LocalNodeId, DisconnectMessage<NodeId>)) -> NoReply {
        if let Some(node) = self.0.get_local_node(&id) {
            let m = RpcMessage::Hyparview(ProtocolMessage::Disconnect(m));
            node.send_rpc_message(m);
        } else {
            println!("# UNKNOWN NODE: {:?}, {:?}", id, m);
            // TODO: metric or log
            // TODO: reply disconnect
        }
        NoReply::done()
    }
}