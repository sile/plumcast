use super::RpcMessage;
use crate::codec::hyparview::{
    DisconnectMessageDecoder, DisconnectMessageEncoder, ForwardJoinMessageDecoder,
    ForwardJoinMessageEncoder, JoinMessageDecoder, JoinMessageEncoder, NeighborMessageDecoder,
    NeighborMessageEncoder, ShuffleMessageDecoder, ShuffleMessageEncoder,
    ShuffleReplyMessageDecoder, ShuffleReplyMessageEncoder,
};
use crate::message::MessagePayload;
use crate::misc::{
    DisconnectMessage, ForwardJoinMessage, JoinMessage, NeighborMessage, ShuffleMessage,
    ShuffleReplyMessage,
};
use crate::node::{LocalNodeId, NodeId};
use crate::service::ServiceHandle;
use crate::Result;
use fibers_rpc::client::ClientServiceHandle;
use fibers_rpc::server::{HandleCast, NoReply, ServerBuilder};
use fibers_rpc::{Cast, ProcedureId};

pub fn register_handlers<M: MessagePayload>(rpc: &mut ServerBuilder, service: &ServiceHandle<M>) {
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

    type Notification = (LocalNodeId, JoinMessage);
    type Decoder = JoinMessageDecoder;
    type Encoder = JoinMessageEncoder;
}

pub fn join_cast(peer: NodeId, m: JoinMessage, service: &ClientServiceHandle) -> Result<()> {
    let mut client = JoinCast::client(&service);
    client.options_mut().force_wakeup = true;
    client.options_mut().priority = 100;
    track!(client.cast(peer.address(), (peer.local_id(), m)))?;
    Ok(())
}

#[derive(Debug)]
struct JoinHandler<M: MessagePayload>(ServiceHandle<M>);
impl<M: MessagePayload> HandleCast<JoinCast> for JoinHandler<M> {
    fn handle_cast(&self, (id, m): (LocalNodeId, JoinMessage)) -> NoReply {
        if let Some(node) = self.0.get_local_node_or_disconnect(id, &m.sender) {
            node.send_rpc_message(RpcMessage::Hyparview(m.into()));
        }
        NoReply::done()
    }
}

#[derive(Debug)]
pub struct ForwardJoinCast;
impl Cast for ForwardJoinCast {
    const ID: ProcedureId = ProcedureId(0x17CC_0001);
    const NAME: &'static str = "hyparview.forward_join";

    type Notification = (LocalNodeId, ForwardJoinMessage);
    type Decoder = ForwardJoinMessageDecoder;
    type Encoder = ForwardJoinMessageEncoder;
}

pub fn forward_join_cast(
    peer: NodeId,
    m: ForwardJoinMessage,
    service: &ClientServiceHandle,
) -> Result<()> {
    let mut client = ForwardJoinCast::client(&service);
    client.options_mut().force_wakeup = true;
    client.options_mut().priority = 100;
    track!(client.cast(peer.address(), (peer.local_id(), m)))?;
    Ok(())
}

#[derive(Debug)]
struct ForwardJoinHandler<M: MessagePayload>(ServiceHandle<M>);
impl<M: MessagePayload> HandleCast<ForwardJoinCast> for ForwardJoinHandler<M> {
    fn handle_cast(&self, (id, m): (LocalNodeId, ForwardJoinMessage)) -> NoReply {
        if let Some(node) = self.0.get_local_node_or_disconnect(id, &m.sender) {
            node.send_rpc_message(RpcMessage::Hyparview(m.into()));
        }
        NoReply::done()
    }
}

#[derive(Debug)]
pub struct NeighborCast;
impl Cast for NeighborCast {
    const ID: ProcedureId = ProcedureId(0x17CC_0002);
    const NAME: &'static str = "hyparview.neighbor";

    type Notification = (LocalNodeId, NeighborMessage);
    type Decoder = NeighborMessageDecoder;
    type Encoder = NeighborMessageEncoder;
}

pub fn neighbor_cast(
    peer: NodeId,
    m: NeighborMessage,
    service: &ClientServiceHandle,
) -> Result<()> {
    let mut client = NeighborCast::client(&service);
    client.options_mut().force_wakeup = true;
    client.options_mut().priority = 100;
    track!(client.cast(peer.address(), (peer.local_id(), m)))?;
    Ok(())
}

#[derive(Debug)]
struct NeighborHandler<M: MessagePayload>(ServiceHandle<M>);
impl<M: MessagePayload> HandleCast<NeighborCast> for NeighborHandler<M> {
    fn handle_cast(&self, (id, m): (LocalNodeId, NeighborMessage)) -> NoReply {
        if let Some(node) = self.0.get_local_node_or_disconnect(id, &m.sender) {
            node.send_rpc_message(RpcMessage::Hyparview(m.into()));
        }
        NoReply::done()
    }
}

#[derive(Debug)]
pub struct ShuffleCast;
impl Cast for ShuffleCast {
    const ID: ProcedureId = ProcedureId(0x17CC_0003);
    const NAME: &'static str = "hyparview.shuffle";

    type Notification = (LocalNodeId, ShuffleMessage);
    type Decoder = ShuffleMessageDecoder;
    type Encoder = ShuffleMessageEncoder;
}

pub fn shuffle_cast(peer: NodeId, m: ShuffleMessage, service: &ClientServiceHandle) -> Result<()> {
    let mut client = ShuffleCast::client(&service);
    client.options_mut().priority = 200;
    track!(client.cast(peer.address(), (peer.local_id(), m)))?;
    Ok(())
}

#[derive(Debug)]
struct ShuffleHandler<M: MessagePayload>(ServiceHandle<M>);
impl<M: MessagePayload> HandleCast<ShuffleCast> for ShuffleHandler<M> {
    fn handle_cast(&self, (id, m): (LocalNodeId, ShuffleMessage)) -> NoReply {
        if let Some(node) = self.0.get_local_node_or_disconnect(id, &m.sender) {
            node.send_rpc_message(RpcMessage::Hyparview(m.into()));
        }
        NoReply::done()
    }
}

#[derive(Debug)]
pub struct ShuffleReplyCast;
impl Cast for ShuffleReplyCast {
    const ID: ProcedureId = ProcedureId(0x17CC_0004);
    const NAME: &'static str = "hyparview.shuffle_reply";

    type Notification = (LocalNodeId, ShuffleReplyMessage);
    type Decoder = ShuffleReplyMessageDecoder;
    type Encoder = ShuffleReplyMessageEncoder;
}

pub fn shuffle_reply_cast(
    peer: NodeId,
    m: ShuffleReplyMessage,
    service: &ClientServiceHandle,
) -> Result<()> {
    let mut client = ShuffleReplyCast::client(&service);
    client.options_mut().priority = 200;
    track!(client.cast(peer.address(), (peer.local_id(), m)))?;
    Ok(())
}

#[derive(Debug)]
struct ShuffleReplyHandler<M: MessagePayload>(ServiceHandle<M>);
impl<M: MessagePayload> HandleCast<ShuffleReplyCast> for ShuffleReplyHandler<M> {
    fn handle_cast(&self, (id, m): (LocalNodeId, ShuffleReplyMessage)) -> NoReply {
        if let Some(node) = self.0.get_local_node_or_disconnect(id, &m.sender) {
            node.send_rpc_message(RpcMessage::Hyparview(m.into()));
        }
        NoReply::done()
    }
}

#[derive(Debug)]
pub struct DisconnectCast;
impl Cast for DisconnectCast {
    const ID: ProcedureId = ProcedureId(0x17CC_0005);
    const NAME: &'static str = "hyparview.disconnect";

    type Notification = (LocalNodeId, DisconnectMessage);
    type Decoder = DisconnectMessageDecoder;
    type Encoder = DisconnectMessageEncoder;
}

pub fn disconnect_cast(
    peer: NodeId,
    m: DisconnectMessage,
    service: &ClientServiceHandle,
) -> Result<()> {
    let client = DisconnectCast::client(&service);
    track!(client.cast(peer.address(), (peer.local_id(), m)))?;
    Ok(())
}

#[derive(Debug)]
struct DisconnectHandler<M: MessagePayload>(ServiceHandle<M>);
impl<M: MessagePayload> HandleCast<DisconnectCast> for DisconnectHandler<M> {
    fn handle_cast(&self, (id, m): (LocalNodeId, DisconnectMessage)) -> NoReply {
        if let Some(node) = self.0.get_local_node(id) {
            node.send_rpc_message(RpcMessage::Hyparview(m.into()));
        }
        NoReply::done()
    }
}
