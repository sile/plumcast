use plumtree;
use std::marker::PhantomData;

use {MessageId, MessagePayload, NodeId};

/// Plumtree node.
pub type PlumtreeNode<M> = plumtree::Node<PlumtreeSystem<M>>;

/// Options for Plumtree nodes.
pub type PlumtreeNodeOptions = plumtree::NodeOptions;

pub type PlumtreeAction<M> = plumtree::Action<PlumtreeSystem<M>>;

pub type Message<M> = plumtree::message::Message<PlumtreeSystem<M>>;
pub type PlumtreeMessage<M> = plumtree::message::ProtocolMessage<PlumtreeSystem<M>>;
pub type GossipMessage<M> = plumtree::message::GossipMessage<PlumtreeSystem<M>>;
pub type GraftMessage<M> = plumtree::message::GraftMessage<PlumtreeSystem<M>>;
pub type IhaveMessage<M> = plumtree::message::IhaveMessage<PlumtreeSystem<M>>;
pub type PruneMessage<M> = plumtree::message::PruneMessage<PlumtreeSystem<M>>;

/// An implementation of `plumtree::System` trait specialised to this crate.
#[derive(Debug)]
pub struct PlumtreeSystem<M>(PhantomData<M>);
impl<M: MessagePayload> plumtree::System for PlumtreeSystem<M> {
    type NodeId = NodeId;
    type MessageId = MessageId;
    type MessagePayload = M;
}
