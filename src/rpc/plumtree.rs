use fibers_rpc::client::ClientServiceHandle;
use fibers_rpc::server::{HandleCast, NoReply, ServerBuilder};
use fibers_rpc::{Cast, ProcedureId};
use plumtree::message::{GossipMessage, GraftMessage, IhaveMessage, ProtocolMessage, PruneMessage};

use super::RpcMessage;
use codec::plumtree::{
    GossipMessageDecoder, GossipMessageEncoder, GraftMessageDecoder, GraftMessageEncoder,
    IhaveMessageDecoder, IhaveMessageEncoder, PruneMessageDecoder, PruneMessageEncoder,
};
use node::System;
use service::ServiceHandle;
use {LocalNodeId, NodeId};

pub fn register_handlers(rpc: &mut ServerBuilder, service: ServiceHandle) {
    rpc.add_cast_handler(GossipHandler(service.clone()));
    rpc.add_cast_handler(IhaveHandler(service.clone()));
    rpc.add_cast_handler(GraftHandler(service.clone()));
    rpc.add_cast_handler(PruneHandler(service.clone()));
}

#[derive(Debug)]
pub struct GossipCast;
impl Cast for GossipCast {
    const ID: ProcedureId = ProcedureId(0x17CD_0000);
    const NAME: &'static str = "plumtree.gossip";

    type Notification = (LocalNodeId, GossipMessage<System>);
    type Decoder = GossipMessageDecoder;
    type Encoder = GossipMessageEncoder;
}

pub fn gossip_cast(peer: NodeId, m: GossipMessage<System>, service: &ClientServiceHandle) {
    // TODO: set options (e.g., priority, force_wakeup)
    let client = GossipCast::client(&service);
    track_try_unwrap!(client.cast(peer.addr, (peer.local_id, m))); // TODO
}

#[derive(Debug)]
struct GossipHandler(ServiceHandle);
impl HandleCast<GossipCast> for GossipHandler {
    fn handle_cast(&self, (id, m): (LocalNodeId, GossipMessage<System>)) -> NoReply {
        if let Some(node) = self.0.get_local_node(&id) {
            let m = RpcMessage::Plumtree(ProtocolMessage::Gossip(m));
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
pub struct IhaveCast;
impl Cast for IhaveCast {
    const ID: ProcedureId = ProcedureId(0x17CD_0001);
    const NAME: &'static str = "plumtree.ihave";

    type Notification = (LocalNodeId, IhaveMessage<System>);
    type Decoder = IhaveMessageDecoder;
    type Encoder = IhaveMessageEncoder;
}

pub fn ihave_cast(peer: NodeId, m: IhaveMessage<System>, service: &ClientServiceHandle) {
    // TODO: set options (e.g., priority, force_wakeup)
    let client = IhaveCast::client(&service);
    track_try_unwrap!(client.cast(peer.addr, (peer.local_id, m))); // TODO
}

#[derive(Debug)]
struct IhaveHandler(ServiceHandle);
impl HandleCast<IhaveCast> for IhaveHandler {
    fn handle_cast(&self, (id, m): (LocalNodeId, IhaveMessage<System>)) -> NoReply {
        if let Some(node) = self.0.get_local_node(&id) {
            let m = RpcMessage::Plumtree(ProtocolMessage::Ihave(m));
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
pub struct GraftCast;
impl Cast for GraftCast {
    const ID: ProcedureId = ProcedureId(0x17CD_0002);
    const NAME: &'static str = "plumtree.graft";

    type Notification = (LocalNodeId, GraftMessage<System>);
    type Decoder = GraftMessageDecoder;
    type Encoder = GraftMessageEncoder;
}

pub fn graft_cast(peer: NodeId, m: GraftMessage<System>, service: &ClientServiceHandle) {
    // TODO: set options (e.g., priority, force_wakeup)
    let client = GraftCast::client(&service);
    track_try_unwrap!(client.cast(peer.addr, (peer.local_id, m))); // TODO
}

#[derive(Debug)]
struct GraftHandler(ServiceHandle);
impl HandleCast<GraftCast> for GraftHandler {
    fn handle_cast(&self, (id, m): (LocalNodeId, GraftMessage<System>)) -> NoReply {
        if let Some(node) = self.0.get_local_node(&id) {
            let m = RpcMessage::Plumtree(ProtocolMessage::Graft(m));
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
pub struct PruneCast;
impl Cast for PruneCast {
    const ID: ProcedureId = ProcedureId(0x17CD_0003);
    const NAME: &'static str = "plumtree.prune";

    type Notification = (LocalNodeId, PruneMessage<System>);
    type Decoder = PruneMessageDecoder;
    type Encoder = PruneMessageEncoder;
}

pub fn prune_cast(peer: NodeId, m: PruneMessage<System>, service: &ClientServiceHandle) {
    // TODO: set options (e.g., priority, force_wakeup)
    let client = PruneCast::client(&service);
    track_try_unwrap!(client.cast(peer.addr, (peer.local_id, m))); // TODO
}

#[derive(Debug)]
struct PruneHandler(ServiceHandle);
impl HandleCast<PruneCast> for PruneHandler {
    fn handle_cast(&self, (id, m): (LocalNodeId, PruneMessage<System>)) -> NoReply {
        if let Some(node) = self.0.get_local_node(&id) {
            let m = RpcMessage::Plumtree(ProtocolMessage::Prune(m));
            node.send_rpc_message(m);
        } else {
            println!("# UNKNOWN NODE: {:?}, {:?}", id, m);
            // TODO: metric or log
            // TODO: reply disconnect
        }
        NoReply::done()
    }
}
