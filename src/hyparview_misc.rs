use hyparview;
use rand::{self, Rng, SeedableRng, StdRng};

use NodeId;

pub type HyparviewAction = hyparview::Action<NodeId>;
pub type HyparviewNode = hyparview::Node<NodeId, StdRng>;

pub type HyparviewMessage = hyparview::message::ProtocolMessage<NodeId>;
pub type DisconnectMessage = hyparview::message::DisconnectMessage<NodeId>;
pub type ForwardJoinMessage = hyparview::message::ForwardJoinMessage<NodeId>;
pub type JoinMessage = hyparview::message::JoinMessage<NodeId>;
pub type NeighborMessage = hyparview::message::NeighborMessage<NodeId>;
pub type ShuffleMessage = hyparview::message::ShuffleMessage<NodeId>;
pub type ShuffleReplyMessage = hyparview::message::ShuffleReplyMessage<NodeId>;

pub fn new_hyparview_node(id: NodeId) -> HyparviewNode {
    let rng = StdRng::from_seed(rand::thread_rng().gen());
    HyparviewNode::with_options(id.clone(), hyparview::NodeOptions::new().set_rng(rng))
}
