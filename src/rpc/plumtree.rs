use fibers_rpc::client::ClientServiceHandle;
use fibers_rpc::server::{HandleCast, NoReply, ServerBuilder};
use fibers_rpc::{Cast, ProcedureId};
use plumtree::message::{GossipMessage, GraftMessage, IhaveMessage, ProtocolMessage, PruneMessage};
use std::marker::PhantomData;

use super::RpcMessage;
use codec::plumtree::{
    GossipMessageDecoder, GossipMessageEncoder, GraftMessageDecoder, GraftMessageEncoder,
    IhaveMessageDecoder, IhaveMessageEncoder, PruneMessageDecoder, PruneMessageEncoder,
};
use node::System;
use service::ServiceHandle;
use {LocalNodeId, MessagePayload, NodeId, Result};

const MAX_QUEUE_LEN: u64 = 4096; // FIXME: parameterize

pub fn register_handlers<M: MessagePayload>(rpc: &mut ServerBuilder, service: ServiceHandle<M>) {
    rpc.add_cast_handler(GossipHandler(service.clone()));
    rpc.add_cast_handler(IhaveHandler(service.clone()));
    rpc.add_cast_handler(GraftHandler(service.clone()));
    rpc.add_cast_handler(PruneHandler(service.clone()));
}

#[derive(Debug)]
pub struct GossipCast<M>(PhantomData<M>);
unsafe impl<M> Sync for GossipCast<M> {}
impl<M: MessagePayload> Cast for GossipCast<M> {
    const ID: ProcedureId = ProcedureId(0x17CD_0000);
    const NAME: &'static str = "plumtree.gossip";

    type Notification = (LocalNodeId, GossipMessage<System<M>>);
    type Decoder = GossipMessageDecoder<M>;
    type Encoder = GossipMessageEncoder<M>;
}

pub fn gossip_cast<M: MessagePayload>(
    peer: NodeId,
    m: GossipMessage<System<M>>,
    service: &ClientServiceHandle,
) -> Result<()> {
    let mut client = GossipCast::client(&service);
    client.options_mut().max_queue_len = Some(MAX_QUEUE_LEN);
    track!(client.cast(peer.address(), (peer.local_id(), m)))?;
    Ok(())
}

#[derive(Debug)]
struct GossipHandler<M: MessagePayload>(ServiceHandle<M>);
impl<M: MessagePayload> HandleCast<GossipCast<M>> for GossipHandler<M> {
    fn handle_cast(&self, (id, m): (LocalNodeId, GossipMessage<System<M>>)) -> NoReply {
        if let Some(node) = self.0.get_local_node_or_disconnect(&id, &m.sender) {
            let m = RpcMessage::Plumtree(ProtocolMessage::Gossip(m));
            node.send_rpc_message(m);
        }
        NoReply::done()
    }
}

#[derive(Debug)]
pub struct IhaveCast<M>(PhantomData<M>);
unsafe impl<M> Sync for IhaveCast<M> {}
impl<M: MessagePayload> Cast for IhaveCast<M> {
    const ID: ProcedureId = ProcedureId(0x17CD_0001);
    const NAME: &'static str = "plumtree.ihave";

    type Notification = (LocalNodeId, IhaveMessage<System<M>>);
    type Decoder = IhaveMessageDecoder<M>;
    type Encoder = IhaveMessageEncoder<M>;
}

pub fn ihave_cast<M: MessagePayload>(
    peer: NodeId,
    m: IhaveMessage<System<M>>,
    service: &ClientServiceHandle,
) -> Result<()> {
    let mut client = IhaveCast::client(&service);
    client.options_mut().priority = 200;
    client.options_mut().max_queue_len = Some(MAX_QUEUE_LEN);
    track!(client.cast(peer.address(), (peer.local_id(), m)))?;
    Ok(())
}

#[derive(Debug)]
struct IhaveHandler<M: MessagePayload>(ServiceHandle<M>);
impl<M: MessagePayload> HandleCast<IhaveCast<M>> for IhaveHandler<M> {
    fn handle_cast(&self, (id, m): (LocalNodeId, IhaveMessage<System<M>>)) -> NoReply {
        if let Some(node) = self.0.get_local_node_or_disconnect(&id, &m.sender) {
            let m = RpcMessage::Plumtree(ProtocolMessage::Ihave(m));
            node.send_rpc_message(m);
        }
        NoReply::done()
    }
}

#[derive(Debug)]
pub struct GraftCast<M>(PhantomData<M>);
unsafe impl<M> Sync for GraftCast<M> {}
impl<M: MessagePayload> Cast for GraftCast<M> {
    const ID: ProcedureId = ProcedureId(0x17CD_0002);
    const NAME: &'static str = "plumtree.graft";

    type Notification = (LocalNodeId, GraftMessage<System<M>>);
    type Decoder = GraftMessageDecoder<M>;
    type Encoder = GraftMessageEncoder<M>;
}

pub fn graft_cast<M: MessagePayload>(
    peer: NodeId,
    m: GraftMessage<System<M>>,
    service: &ClientServiceHandle,
) -> Result<()> {
    let client = GraftCast::client(&service);
    track!(client.cast(peer.address(), (peer.local_id(), m)))?;
    Ok(())
}

#[derive(Debug)]
struct GraftHandler<M: MessagePayload>(ServiceHandle<M>);
impl<M: MessagePayload> HandleCast<GraftCast<M>> for GraftHandler<M> {
    fn handle_cast(&self, (id, m): (LocalNodeId, GraftMessage<System<M>>)) -> NoReply {
        if let Some(node) = self.0.get_local_node_or_disconnect(&id, &m.sender) {
            let m = RpcMessage::Plumtree(ProtocolMessage::Graft(m));
            node.send_rpc_message(m);
        }
        NoReply::done()
    }
}

#[derive(Debug)]
pub struct PruneCast<M>(PhantomData<M>);
unsafe impl<M> Sync for PruneCast<M> {}
impl<M: MessagePayload> Cast for PruneCast<M> {
    const ID: ProcedureId = ProcedureId(0x17CD_0003);
    const NAME: &'static str = "plumtree.prune";

    type Notification = (LocalNodeId, PruneMessage<System<M>>);
    type Decoder = PruneMessageDecoder<M>;
    type Encoder = PruneMessageEncoder<M>;
}

pub fn prune_cast<M: MessagePayload>(
    peer: NodeId,
    m: PruneMessage<System<M>>,
    service: &ClientServiceHandle,
) -> Result<()> {
    let client = PruneCast::client(&service);
    track!(client.cast(peer.address(), (peer.local_id(), m)))?;
    Ok(())
}

#[derive(Debug)]
struct PruneHandler<M: MessagePayload>(ServiceHandle<M>);
impl<M: MessagePayload> HandleCast<PruneCast<M>> for PruneHandler<M> {
    fn handle_cast(&self, (id, m): (LocalNodeId, PruneMessage<System<M>>)) -> NoReply {
        if let Some(node) = self.0.get_local_node_or_disconnect(&id, &m.sender) {
            let m = RpcMessage::Plumtree(ProtocolMessage::Prune(m));
            node.send_rpc_message(m);
        }
        NoReply::done()
    }
}
