use hyparview;
use rand::StdRng;

use NodeId;

pub type HyparviewAction = hyparview::Action<NodeId>;
pub type HyparviewNode = hyparview::Node<NodeId, StdRng>;
pub type HyparviewNodeOptions = hyparview::NodeOptions;

pub type HyparviewMessage = hyparview::message::ProtocolMessage<NodeId>;
pub type DisconnectMessage = hyparview::message::DisconnectMessage<NodeId>;
pub type ForwardJoinMessage = hyparview::message::ForwardJoinMessage<NodeId>;
pub type JoinMessage = hyparview::message::JoinMessage<NodeId>;
pub type NeighborMessage = hyparview::message::NeighborMessage<NodeId>;
pub type ShuffleMessage = hyparview::message::ShuffleMessage<NodeId>;
pub type ShuffleReplyMessage = hyparview::message::ShuffleReplyMessage<NodeId>;
