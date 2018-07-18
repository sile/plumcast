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
use {LocalNodeId, MessagePayload, NodeId, Result};

pub fn register_handlers<M: MessagePayload>(rpc: &mut ServerBuilder, service: ServiceHandle<M>) {
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

pub fn join_cast(
    peer: NodeId,
    m: JoinMessage<NodeId>,
    service: &ClientServiceHandle,
) -> Result<()> {
    let mut client = JoinCast::client(&service);
    client.options_mut().force_wakeup = true;
    client.options_mut().priority = 100;
    track!(client.cast(peer.addr, (peer.local_id, m)))?;
    Ok(())
}

#[derive(Debug)]
struct JoinHandler<M: MessagePayload>(ServiceHandle<M>);
impl<M: MessagePayload> HandleCast<JoinCast> for JoinHandler<M> {
    fn handle_cast(&self, (id, m): (LocalNodeId, JoinMessage<NodeId>)) -> NoReply {
        if let Some(node) = self.0.get_local_node_or_disconnect(&id, &m.sender) {
            let m = RpcMessage::Hyparview(ProtocolMessage::Join(m));
            node.send_rpc_message(m);
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
) -> Result<()> {
    let mut client = ForwardJoinCast::client(&service);
    client.options_mut().force_wakeup = true;
    client.options_mut().priority = 100;
    track!(client.cast(peer.addr, (peer.local_id, m)))?;
    Ok(())
}

#[derive(Debug)]
struct ForwardJoinHandler<M: MessagePayload>(ServiceHandle<M>);
impl<M: MessagePayload> HandleCast<ForwardJoinCast> for ForwardJoinHandler<M> {
    fn handle_cast(&self, (id, m): (LocalNodeId, ForwardJoinMessage<NodeId>)) -> NoReply {
        if let Some(node) = self.0.get_local_node_or_disconnect(&id, &m.sender) {
            let m = RpcMessage::Hyparview(ProtocolMessage::ForwardJoin(m));
            node.send_rpc_message(m);
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

pub fn neighbor_cast(
    peer: NodeId,
    m: NeighborMessage<NodeId>,
    service: &ClientServiceHandle,
) -> Result<()> {
    let mut client = NeighborCast::client(&service);
    if m.high_priority {
        client.options_mut().force_wakeup = true;
        client.options_mut().priority = 100;
    }
    track!(client.cast(peer.addr, (peer.local_id, m)))?;
    Ok(())
}

#[derive(Debug)]
struct NeighborHandler<M: MessagePayload>(ServiceHandle<M>);
impl<M: MessagePayload> HandleCast<NeighborCast> for NeighborHandler<M> {
    fn handle_cast(&self, (id, m): (LocalNodeId, NeighborMessage<NodeId>)) -> NoReply {
        if let Some(node) = self.0.get_local_node_or_disconnect(&id, &m.sender) {
            let m = RpcMessage::Hyparview(ProtocolMessage::Neighbor(m));
            node.send_rpc_message(m);
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

pub fn shuffle_cast(
    peer: NodeId,
    m: ShuffleMessage<NodeId>,
    service: &ClientServiceHandle,
) -> Result<()> {
    let mut client = ShuffleCast::client(&service);
    client.options_mut().priority = 200;
    track!(client.cast(peer.addr, (peer.local_id, m)))?;
    Ok(())
}

#[derive(Debug)]
struct ShuffleHandler<M: MessagePayload>(ServiceHandle<M>);
impl<M: MessagePayload> HandleCast<ShuffleCast> for ShuffleHandler<M> {
    fn handle_cast(&self, (id, m): (LocalNodeId, ShuffleMessage<NodeId>)) -> NoReply {
        if let Some(node) = self.0.get_local_node_or_disconnect(&id, &m.sender) {
            let m = RpcMessage::Hyparview(ProtocolMessage::Shuffle(m));
            node.send_rpc_message(m);
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
) -> Result<()> {
    let mut client = ShuffleReplyCast::client(&service);
    client.options_mut().priority = 200;
    track!(client.cast(peer.addr, (peer.local_id, m)))?;
    Ok(())
}

#[derive(Debug)]
struct ShuffleReplyHandler<M: MessagePayload>(ServiceHandle<M>);
impl<M: MessagePayload> HandleCast<ShuffleReplyCast> for ShuffleReplyHandler<M> {
    fn handle_cast(&self, (id, m): (LocalNodeId, ShuffleReplyMessage<NodeId>)) -> NoReply {
        if let Some(node) = self.0.get_local_node_or_disconnect(&id, &m.sender) {
            let m = RpcMessage::Hyparview(ProtocolMessage::ShuffleReply(m));
            node.send_rpc_message(m);
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

pub fn disconnect_cast(
    peer: NodeId,
    m: DisconnectMessage<NodeId>,
    service: &ClientServiceHandle,
) -> Result<()> {
    let client = DisconnectCast::client(&service);
    track!(client.cast(peer.addr, (peer.local_id, m)))?;
    Ok(())
}

#[derive(Debug)]
struct DisconnectHandler<M: MessagePayload>(ServiceHandle<M>);
impl<M: MessagePayload> HandleCast<DisconnectCast> for DisconnectHandler<M> {
    fn handle_cast(&self, (id, m): (LocalNodeId, DisconnectMessage<NodeId>)) -> NoReply {
        if let Some(node) = self.0.get_local_node(&id) {
            let m = RpcMessage::Hyparview(ProtocolMessage::Disconnect(m));
            node.send_rpc_message(m);
        }
        NoReply::done()
    }
}
