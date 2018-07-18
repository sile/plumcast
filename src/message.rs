use bytecodec::bytes::{BytesEncoder, RemainingBytesDecoder};
use bytecodec::{Decode, Encode};
use plumtree::message::Message as InnerMessage;

use node::System;
use NodeId;

/// Broadcasted application message.
#[derive(Debug, Clone)]
pub struct Message<T: MessagePayload>(InnerMessage<System<T>>);
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

    pub(crate) fn new(m: InnerMessage<System<T>>) -> Self {
        Message(m)
    }
}

/// The identifier of a message.
///
/// TODO: doc of node part and seqno part
/// TODO: doc of generation rule (only generated crate internally)
/// TODO: doc; uniquness grantee of identifiers
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
