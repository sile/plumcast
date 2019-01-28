//! [`Message`] and related components.
//!
//! [`Message`]: ./struct.Message.html
use crate::misc::PlumtreeAppMessage;
use crate::node::NodeId;
use bytecodec::bytes::{BytesEncoder, RemainingBytesDecoder, Utf8Decoder, Utf8Encoder};
use bytecodec::{Decode, Encode};

/// Broadcasted application message.
#[derive(Debug, Clone)]
pub struct Message<T: MessagePayload>(PlumtreeAppMessage<T>);
impl<T: MessagePayload> Message<T> {
    /// Returns a reference to the identifier of the message.
    pub fn id(&self) -> &MessageId {
        &self.0.id
    }

    /// Returns a reference to the payload of the message.
    pub fn payload(&self) -> &T {
        &self.0.payload
    }

    /// Returns a mutable reference to the payload of the message.
    pub fn payload_mut(&mut self) -> &mut T {
        &mut self.0.payload
    }

    /// Takes the ownership of the message, and returns its payload.
    pub fn into_payload(self) -> T {
        self.0.payload
    }

    pub(crate) fn new(message: PlumtreeAppMessage<T>) -> Self {
        Message(message)
    }
}

/// Message identifier.
///
/// An identifier consists of the node identifier part and the sequence number part.
/// The node identifier part which type is [`NodeId`] indicates the sender (origin) of the message.
/// The sequence number part indicates the number of messages broadcasted by the sender so far.
///
/// Identifiers are assigned automatically when broadcasting messages.
///
/// It is guaranteed that the identifiers are unique in a cluster
/// unless the OS processes executing plumcast nodes are restarted.
/// Practically confliction of identifiers is extremely rare
/// even if OS processes are frequently restarted.
///
/// [`NodeId`]: ../node/struct.NodeId.html
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MessageId {
    node: NodeId,
    seqno: u64,
}
impl MessageId {
    /// Returns the node identifier part of the message identifier.
    pub fn node(&self) -> &NodeId {
        &self.node
    }

    /// Returns the sequence number part of the message identifier.
    pub fn seqno(&self) -> u64 {
        self.seqno
    }

    pub(crate) fn new(node: NodeId, seqno: u64) -> Self {
        MessageId { node, seqno }
    }
}

/// This trait allows the implementations to be used as the payload of broadcasting messages.
pub trait MessagePayload: Sized + Clone + Send + 'static {
    /// Payload encoder.
    ///
    /// This is used to serialize payload for transmitting to remote nodes.
    type Encoder: Encode<Item = Self> + Default + Send + 'static;

    /// Payload decoder.
    ///
    /// This is used to deserialize payload from octets received from remote nodes.
    type Decoder: Decode<Item = Self> + Default + Send + 'static;
}
impl MessagePayload for Vec<u8> {
    type Encoder = BytesEncoder<Vec<u8>>;
    type Decoder = RemainingBytesDecoder;
}
impl MessagePayload for String {
    type Encoder = Utf8Encoder;
    type Decoder = Utf8Decoder;
}
