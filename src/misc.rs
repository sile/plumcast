//! Miscellaneous components.
use crate::message::{MessageId, MessagePayload};
use crate::node::NodeId;
use fibers::Spawn;
use futures::Future;
use hyparview;
use plumtree;
use rand::rngs::StdRng;
use std::fmt;
use std::marker::PhantomData;
use std::sync::Arc;

type ArcFn = Arc<dyn Fn(Box<dyn Future<Item = (), Error = ()> + Send>) + Send + Sync + 'static>;

/// Sharable [`Spawn`].
///
/// [`Spawn`]: https://docs.rs/fibers/0.1/fibers/trait.Spawn.html
#[derive(Clone)]
pub struct ArcSpawn(ArcFn);
impl ArcSpawn {
    pub(crate) fn new<S>(inner: S) -> Self
    where
        S: Spawn + Send + Sync + 'static,
    {
        ArcSpawn(Arc::new(move |fiber| inner.spawn_boxed(fiber)))
    }
}
impl Spawn for ArcSpawn {
    fn spawn_boxed(&self, fiber: Box<dyn Future<Item = (), Error = ()> + Send>) {
        (self.0)(fiber);
    }
}
impl fmt::Debug for ArcSpawn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ArcSpawn(_)")
    }
}

/// HyParView node.
pub type HyparviewNode = hyparview::Node<NodeId, StdRng>;

/// Options for HyParView nodes.
pub type HyparviewNodeOptions = hyparview::NodeOptions;

pub(crate) type HyparviewAction = hyparview::Action<NodeId>;

pub(crate) type HyparviewMessage = hyparview::message::ProtocolMessage<NodeId>;
pub(crate) type DisconnectMessage = hyparview::message::DisconnectMessage<NodeId>;
pub(crate) type ForwardJoinMessage = hyparview::message::ForwardJoinMessage<NodeId>;
pub(crate) type JoinMessage = hyparview::message::JoinMessage<NodeId>;
pub(crate) type NeighborMessage = hyparview::message::NeighborMessage<NodeId>;
pub(crate) type ShuffleMessage = hyparview::message::ShuffleMessage<NodeId>;
pub(crate) type ShuffleReplyMessage = hyparview::message::ShuffleReplyMessage<NodeId>;

/// Plumtree node.
pub type PlumtreeNode<M> = plumtree::Node<PlumtreeSystem<M>>;

/// Options for Plumtree nodes.
pub type PlumtreeNodeOptions = plumtree::NodeOptions;

pub(crate) type PlumtreeAction<M> = plumtree::Action<PlumtreeSystem<M>>;
pub(crate) type PlumtreeAppMessage<M> = plumtree::message::Message<PlumtreeSystem<M>>;
pub(crate) type PlumtreeMessage<M> = plumtree::message::ProtocolMessage<PlumtreeSystem<M>>;
pub(crate) type GossipMessage<M> = plumtree::message::GossipMessage<PlumtreeSystem<M>>;
pub(crate) type GraftMessage<M> = plumtree::message::GraftMessage<PlumtreeSystem<M>>;
pub(crate) type IhaveMessage<M> = plumtree::message::IhaveMessage<PlumtreeSystem<M>>;
pub(crate) type PruneMessage<M> = plumtree::message::PruneMessage<PlumtreeSystem<M>>;

/// An implementation of [`plumtree::System`] trait specialised to this crate.
///
/// [`plumtree::System`]: https://docs.rs/plumtree/0.1/plumtree/trait.System.html
#[derive(Debug)]
pub struct PlumtreeSystem<M>(PhantomData<M>);
impl<M: MessagePayload> plumtree::System for PlumtreeSystem<M> {
    type NodeId = NodeId;
    type MessageId = MessageId;
    type MessagePayload = M;
}
