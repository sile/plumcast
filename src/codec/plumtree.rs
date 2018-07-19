use bytecodec::bytes::{BytesDecoder, BytesEncoder};
use bytecodec::combinator::Peekable;
use bytecodec::fixnum::{
    U16beDecoder, U16beEncoder, U32beDecoder, U32beEncoder, U64beDecoder, U64beEncoder, U8Decoder,
    U8Encoder,
};
use bytecodec::{ByteCount, Decode, Encode, Eos, ErrorKind, Result, SizedEncode};
use std::fmt;
use std::marker::PhantomData;

use super::node::{LocalNodeIdDecoder, LocalNodeIdEncoder, NodeIdDecoder, NodeIdEncoder};
use plumtree_misc::{GossipMessage, GraftMessage, IhaveMessage, Message, PruneMessage};
use {LocalNodeId, MessageId, MessagePayload};

pub struct GossipMessageDecoder<M: MessagePayload> {
    destination: LocalNodeIdDecoder,
    sender: NodeIdDecoder,
    round: U16beDecoder,
    message: MessageDecoder<M>,
}
impl<M: MessagePayload> Default for GossipMessageDecoder<M> {
    fn default() -> Self {
        GossipMessageDecoder {
            destination: Default::default(),
            sender: Default::default(),
            round: Default::default(),
            message: Default::default(),
        }
    }
}
impl<M: MessagePayload> fmt::Debug for GossipMessageDecoder<M>
where
    M::Decoder: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "GossipMessageDecoder {{ destination: {:?}, sender: {:?}, \
             round: {:?}, message: {:?} }}",
            self.destination, self.sender, self.round, self.message
        )
    }
}
impl<M: MessagePayload> Decode for GossipMessageDecoder<M> {
    type Item = (LocalNodeId, GossipMessage<M>);

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        bytecodec_try_decode!(self.destination, offset, buf, eos);
        bytecodec_try_decode!(self.sender, offset, buf, eos);
        bytecodec_try_decode!(self.round, offset, buf, eos);
        bytecodec_try_decode!(self.message, offset, buf, eos);
        Ok(offset)
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        let destination = track!(self.destination.finish_decoding())?;
        let sender = track!(self.sender.finish_decoding())?;
        let round = track!(self.round.finish_decoding())?;
        let message = track!(self.message.finish_decoding())?;
        let gossip = GossipMessage {
            sender,
            round,
            message,
        };
        Ok((destination, gossip))
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.destination
            .requiring_bytes()
            .add_for_decoding(self.sender.requiring_bytes())
            .add_for_decoding(self.round.requiring_bytes())
            .add_for_decoding(self.message.requiring_bytes())
    }

    fn is_idle(&self) -> bool {
        self.round.is_idle() && self.message.is_idle()
    }
}

struct MessageDecoder<M: MessagePayload> {
    id: MessageIdDecoder,
    payload: M::Decoder,
}
impl<M: MessagePayload> Default for MessageDecoder<M> {
    fn default() -> Self {
        MessageDecoder {
            id: Default::default(),
            payload: Default::default(),
        }
    }
}
impl<M: MessagePayload> fmt::Debug for MessageDecoder<M>
where
    M::Decoder: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "MessageDecoder {{ id: {:?}, payload: {:?} }}",
            self.id, self.payload
        )
    }
}
impl<M: MessagePayload> Decode for MessageDecoder<M> {
    type Item = Message<M>;

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        bytecodec_try_decode!(self.id, offset, buf, eos);
        bytecodec_try_decode!(self.payload, offset, buf, eos);
        Ok(offset)
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        let id = track!(self.id.finish_decoding())?;
        let payload = track!(self.payload.finish_decoding())?;
        Ok(Message { id, payload })
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.id
            .requiring_bytes()
            .add_for_decoding(self.payload.requiring_bytes())
    }

    fn is_idle(&self) -> bool {
        self.id.is_idle() && self.payload.is_idle()
    }
}

#[derive(Debug, Default)]
struct MessageIdDecoder {
    node: NodeIdDecoder,
    seqno: U64beDecoder,
}
impl Decode for MessageIdDecoder {
    type Item = MessageId;

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        bytecodec_try_decode!(self.node, offset, buf, eos);
        bytecodec_try_decode!(self.seqno, offset, buf, eos);
        Ok(offset)
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        let node = track!(self.node.finish_decoding())?;
        let seqno = track!(self.seqno.finish_decoding())?;
        Ok(MessageId::new(node, seqno))
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.node
            .requiring_bytes()
            .add_for_decoding(self.seqno.requiring_bytes())
    }

    fn is_idle(&self) -> bool {
        self.seqno.is_idle()
    }
}

#[derive(Debug, Default)]
struct MessagePayloadDecoder {
    size: Peekable<U32beDecoder>,
    data: BytesDecoder<Vec<u8>>,
}
impl Decode for MessagePayloadDecoder {
    type Item = Vec<u8>;

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        if !self.size.is_idle() {
            bytecodec_try_decode!(self.size, offset, buf, eos);

            let size = self.size.peek().cloned().expect("Never fails");
            let buf = vec![0; size as usize];
            self.data.set_bytes(buf);
        }
        bytecodec_try_decode!(self.data, offset, buf, eos);
        Ok(offset)
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        let _ = track!(self.size.finish_decoding())?;
        let data = track!(self.data.finish_decoding())?;
        Ok(data)
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.size
            .requiring_bytes()
            .add_for_decoding(self.data.requiring_bytes())
    }

    fn is_idle(&self) -> bool {
        self.size.is_idle() && self.data.is_idle()
    }
}

pub struct GossipMessageEncoder<M: MessagePayload> {
    destination: LocalNodeIdEncoder,
    sender: NodeIdEncoder,
    round: U16beEncoder,
    message: MessageEncoder<M>,
}
impl<M: MessagePayload> Default for GossipMessageEncoder<M> {
    fn default() -> Self {
        GossipMessageEncoder {
            destination: Default::default(),
            sender: Default::default(),
            round: Default::default(),
            message: Default::default(),
        }
    }
}
impl<M: MessagePayload> fmt::Debug for GossipMessageEncoder<M>
where
    M::Encoder: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "GossipMessageEncoder {{ destination: {:?}, sender: {:?}, \
             round: {:?}, message: {:?} }}",
            self.destination, self.sender, self.round, self.message
        )
    }
}
impl<M: MessagePayload> Encode for GossipMessageEncoder<M> {
    type Item = (LocalNodeId, GossipMessage<M>);

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        bytecodec_try_encode!(self.destination, offset, buf, eos);
        bytecodec_try_encode!(self.sender, offset, buf, eos);
        bytecodec_try_encode!(self.round, offset, buf, eos);
        bytecodec_try_encode!(self.message, offset, buf, eos);
        Ok(offset)
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track!(self.destination.start_encoding(item.0))?;
        track!(self.sender.start_encoding(item.1.sender))?;
        track!(self.round.start_encoding(item.1.round))?;
        track!(self.message.start_encoding(item.1.message))?;
        Ok(())
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.destination
            .requiring_bytes()
            .add_for_encoding(self.sender.requiring_bytes())
            .add_for_encoding(self.round.requiring_bytes())
            .add_for_encoding(self.message.requiring_bytes())
    }

    fn is_idle(&self) -> bool {
        self.round.is_idle() && self.message.is_idle()
    }
}
impl<M: MessagePayload> SizedEncode for GossipMessageEncoder<M>
where
    M::Encoder: SizedEncode,
{
    fn exact_requiring_bytes(&self) -> u64 {
        self.destination.exact_requiring_bytes()
            + self.sender.exact_requiring_bytes()
            + self.round.exact_requiring_bytes()
            + self.message.exact_requiring_bytes()
    }
}

struct MessageEncoder<M: MessagePayload> {
    id: MessageIdEncoder,
    payload: M::Encoder,
}
impl<M: MessagePayload> Default for MessageEncoder<M> {
    fn default() -> Self {
        MessageEncoder {
            id: Default::default(),
            payload: Default::default(),
        }
    }
}
impl<M: MessagePayload> fmt::Debug for MessageEncoder<M>
where
    M::Encoder: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "MessageEncoder {{ id: {:?}, payload: {:?} }}",
            self.id, self.payload
        )
    }
}
impl<M: MessagePayload> Encode for MessageEncoder<M> {
    type Item = Message<M>;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        bytecodec_try_encode!(self.id, offset, buf, eos);
        bytecodec_try_encode!(self.payload, offset, buf, eos);
        Ok(offset)
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track!(self.id.start_encoding(item.id))?;
        track!(self.payload.start_encoding(item.payload))?;
        Ok(())
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.id
            .requiring_bytes()
            .add_for_encoding(self.payload.requiring_bytes())
    }

    fn is_idle(&self) -> bool {
        self.id.is_idle() && self.payload.is_idle()
    }
}
impl<M: MessagePayload> SizedEncode for MessageEncoder<M>
where
    M::Encoder: SizedEncode,
{
    fn exact_requiring_bytes(&self) -> u64 {
        self.id.exact_requiring_bytes() + self.payload.exact_requiring_bytes()
    }
}

#[derive(Debug, Default)]
struct MessageIdEncoder {
    node: NodeIdEncoder,
    seqno: U64beEncoder,
}
impl Encode for MessageIdEncoder {
    type Item = MessageId;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        bytecodec_try_encode!(self.node, offset, buf, eos);
        bytecodec_try_encode!(self.seqno, offset, buf, eos);
        Ok(offset)
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track!(self.node.start_encoding(item.node().clone()))?;
        track!(self.seqno.start_encoding(item.seqno()))?;
        Ok(())
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.node
            .requiring_bytes()
            .add_for_encoding(self.seqno.requiring_bytes())
    }

    fn is_idle(&self) -> bool {
        self.seqno.is_idle()
    }
}
impl SizedEncode for MessageIdEncoder {
    fn exact_requiring_bytes(&self) -> u64 {
        self.node.exact_requiring_bytes() + self.seqno.exact_requiring_bytes()
    }
}

#[derive(Debug, Default)]
struct MessagePayloadEncoder {
    size: U32beEncoder,
    data: BytesEncoder<Vec<u8>>,
}
impl Encode for MessagePayloadEncoder {
    type Item = Vec<u8>;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        bytecodec_try_encode!(self.size, offset, buf, eos);
        bytecodec_try_encode!(self.data, offset, buf, eos);
        Ok(offset)
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track!(self.size.start_encoding(item.len() as u32))?;
        track!(self.data.start_encoding(item))?;
        Ok(())
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.size
            .requiring_bytes()
            .add_for_encoding(self.data.requiring_bytes())
    }

    fn is_idle(&self) -> bool {
        self.size.is_idle() && self.data.is_idle()
    }
}
impl SizedEncode for MessagePayloadEncoder {
    fn exact_requiring_bytes(&self) -> u64 {
        self.size.exact_requiring_bytes() + self.data.exact_requiring_bytes()
    }
}

#[derive(Debug)]
pub struct IhaveMessageDecoder<M> {
    destination: LocalNodeIdDecoder,
    sender: NodeIdDecoder,
    round: U16beDecoder,
    message_id: MessageIdDecoder,
    realtime: U8Decoder,
    _phantom: PhantomData<M>,
}
impl<M> Default for IhaveMessageDecoder<M> {
    fn default() -> Self {
        IhaveMessageDecoder {
            destination: Default::default(),
            sender: Default::default(),
            round: Default::default(),
            message_id: Default::default(),
            realtime: Default::default(),
            _phantom: PhantomData,
        }
    }
}
impl<M: MessagePayload> Decode for IhaveMessageDecoder<M> {
    type Item = (LocalNodeId, IhaveMessage<M>);

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        bytecodec_try_decode!(self.destination, offset, buf, eos);
        bytecodec_try_decode!(self.sender, offset, buf, eos);
        bytecodec_try_decode!(self.round, offset, buf, eos);
        bytecodec_try_decode!(self.message_id, offset, buf, eos);
        bytecodec_try_decode!(self.realtime, offset, buf, eos);
        Ok(offset)
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        let destination = track!(self.destination.finish_decoding())?;
        let sender = track!(self.sender.finish_decoding())?;
        let round = track!(self.round.finish_decoding())?;
        let message_id = track!(self.message_id.finish_decoding())?;
        let realtime = track!(self.realtime.finish_decoding())?;

        let message = IhaveMessage {
            sender,
            round,
            message_id,
            realtime: realtime != 0,
        };
        Ok((destination, message))
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.destination
            .requiring_bytes()
            .add_for_decoding(self.sender.requiring_bytes())
            .add_for_decoding(self.round.requiring_bytes())
            .add_for_decoding(self.message_id.requiring_bytes())
            .add_for_decoding(self.realtime.requiring_bytes())
    }

    fn is_idle(&self) -> bool {
        self.realtime.is_idle()
    }
}

#[derive(Debug)]
pub struct IhaveMessageEncoder<M> {
    destination: LocalNodeIdEncoder,
    sender: NodeIdEncoder,
    round: U16beEncoder,
    message_id: MessageIdEncoder,
    realtime: U8Encoder,
    _phantom: PhantomData<M>,
}
impl<M> Default for IhaveMessageEncoder<M> {
    fn default() -> Self {
        IhaveMessageEncoder {
            destination: Default::default(),
            sender: Default::default(),
            round: Default::default(),
            message_id: Default::default(),
            realtime: Default::default(),
            _phantom: PhantomData,
        }
    }
}
impl<M: MessagePayload> Encode for IhaveMessageEncoder<M> {
    type Item = (LocalNodeId, IhaveMessage<M>);

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        bytecodec_try_encode!(self.destination, offset, buf, eos);
        bytecodec_try_encode!(self.sender, offset, buf, eos);
        bytecodec_try_encode!(self.round, offset, buf, eos);
        bytecodec_try_encode!(self.message_id, offset, buf, eos);
        bytecodec_try_encode!(self.realtime, offset, buf, eos);
        Ok(offset)
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track!(self.destination.start_encoding(item.0))?;
        track!(self.sender.start_encoding(item.1.sender))?;
        track!(self.round.start_encoding(item.1.round))?;
        track!(self.message_id.start_encoding(item.1.message_id))?;
        track!(self.realtime.start_encoding(item.1.realtime as u8))?;
        Ok(())
    }

    fn requiring_bytes(&self) -> ByteCount {
        ByteCount::Finite(self.exact_requiring_bytes())
    }

    fn is_idle(&self) -> bool {
        self.realtime.is_idle()
    }
}
impl<M: MessagePayload> SizedEncode for IhaveMessageEncoder<M> {
    fn exact_requiring_bytes(&self) -> u64 {
        self.destination.exact_requiring_bytes()
            + self.sender.exact_requiring_bytes()
            + self.round.exact_requiring_bytes()
            + self.message_id.exact_requiring_bytes()
            + self.realtime.exact_requiring_bytes()
    }
}

#[derive(Debug)]
pub struct GraftMessageDecoder<M> {
    destination: LocalNodeIdDecoder,
    sender: NodeIdDecoder,
    round: U16beDecoder,
    message_id: MessageIdDecoder,
    _phantom: PhantomData<M>,
}
impl<M> Default for GraftMessageDecoder<M> {
    fn default() -> Self {
        GraftMessageDecoder {
            destination: Default::default(),
            sender: Default::default(),
            round: Default::default(),
            message_id: Default::default(),
            _phantom: PhantomData,
        }
    }
}
impl<M: MessagePayload> Decode for GraftMessageDecoder<M> {
    type Item = (LocalNodeId, GraftMessage<M>);

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        bytecodec_try_decode!(self.destination, offset, buf, eos);
        bytecodec_try_decode!(self.sender, offset, buf, eos);
        bytecodec_try_decode!(self.round, offset, buf, eos);
        bytecodec_try_decode!(self.message_id, offset, buf, eos);
        Ok(offset)
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        let destination = track!(self.destination.finish_decoding())?;
        let sender = track!(self.sender.finish_decoding())?;
        let round = track!(self.round.finish_decoding())?;
        let message_id = track!(self.message_id.finish_decoding())?;

        let message = GraftMessage {
            sender,
            round,
            message_id: Some(message_id),
        };
        Ok((destination, message))
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.destination
            .requiring_bytes()
            .add_for_decoding(self.sender.requiring_bytes())
            .add_for_decoding(self.round.requiring_bytes())
            .add_for_decoding(self.message_id.requiring_bytes())
    }

    fn is_idle(&self) -> bool {
        self.message_id.is_idle()
    }
}

#[derive(Debug)]
pub struct GraftMessageEncoder<M> {
    destination: LocalNodeIdEncoder,
    sender: NodeIdEncoder,
    round: U16beEncoder,
    message_id: MessageIdEncoder,
    _phantom: PhantomData<M>,
}
impl<M> Default for GraftMessageEncoder<M> {
    fn default() -> Self {
        GraftMessageEncoder {
            destination: Default::default(),
            sender: Default::default(),
            round: Default::default(),
            message_id: Default::default(),
            _phantom: PhantomData,
        }
    }
}
impl<M: MessagePayload> Encode for GraftMessageEncoder<M> {
    type Item = (LocalNodeId, GraftMessage<M>);

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        bytecodec_try_encode!(self.destination, offset, buf, eos);
        bytecodec_try_encode!(self.sender, offset, buf, eos);
        bytecodec_try_encode!(self.round, offset, buf, eos);
        bytecodec_try_encode!(self.message_id, offset, buf, eos);
        Ok(offset)
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track!(self.destination.start_encoding(item.0))?;
        track!(self.sender.start_encoding(item.1.sender))?;
        track!(self.round.start_encoding(item.1.round))?;

        let message_id = track_assert_some!(item.1.message_id, ErrorKind::InconsistentState);
        track!(self.message_id.start_encoding(message_id))?;
        Ok(())
    }

    fn requiring_bytes(&self) -> ByteCount {
        ByteCount::Finite(self.exact_requiring_bytes())
    }

    fn is_idle(&self) -> bool {
        self.message_id.is_idle()
    }
}
impl<M: MessagePayload> SizedEncode for GraftMessageEncoder<M> {
    fn exact_requiring_bytes(&self) -> u64 {
        self.destination.exact_requiring_bytes()
            + self.sender.exact_requiring_bytes()
            + self.round.exact_requiring_bytes()
            + self.message_id.exact_requiring_bytes()
    }
}

#[derive(Debug)]
pub struct GraftOptimizeMessageDecoder<M> {
    destination: LocalNodeIdDecoder,
    sender: NodeIdDecoder,
    round: U16beDecoder,
    _phantom: PhantomData<M>,
}
impl<M> Default for GraftOptimizeMessageDecoder<M> {
    fn default() -> Self {
        GraftOptimizeMessageDecoder {
            destination: Default::default(),
            sender: Default::default(),
            round: Default::default(),
            _phantom: PhantomData,
        }
    }
}
impl<M: MessagePayload> Decode for GraftOptimizeMessageDecoder<M> {
    type Item = (LocalNodeId, GraftMessage<M>);

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        bytecodec_try_decode!(self.destination, offset, buf, eos);
        bytecodec_try_decode!(self.sender, offset, buf, eos);
        bytecodec_try_decode!(self.round, offset, buf, eos);
        Ok(offset)
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        let destination = track!(self.destination.finish_decoding())?;
        let sender = track!(self.sender.finish_decoding())?;
        let round = track!(self.round.finish_decoding())?;

        let message = GraftMessage {
            sender,
            round,
            message_id: None,
        };
        Ok((destination, message))
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.destination
            .requiring_bytes()
            .add_for_decoding(self.sender.requiring_bytes())
            .add_for_decoding(self.round.requiring_bytes())
    }

    fn is_idle(&self) -> bool {
        self.round.is_idle()
    }
}

#[derive(Debug)]
pub struct GraftOptimizeMessageEncoder<M> {
    destination: LocalNodeIdEncoder,
    sender: NodeIdEncoder,
    round: U16beEncoder,
    _phantom: PhantomData<M>,
}
impl<M> Default for GraftOptimizeMessageEncoder<M> {
    fn default() -> Self {
        GraftOptimizeMessageEncoder {
            destination: Default::default(),
            sender: Default::default(),
            round: Default::default(),
            _phantom: PhantomData,
        }
    }
}
impl<M: MessagePayload> Encode for GraftOptimizeMessageEncoder<M> {
    type Item = (LocalNodeId, GraftMessage<M>);

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        bytecodec_try_encode!(self.destination, offset, buf, eos);
        bytecodec_try_encode!(self.sender, offset, buf, eos);
        bytecodec_try_encode!(self.round, offset, buf, eos);
        Ok(offset)
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track!(self.destination.start_encoding(item.0))?;
        track!(self.sender.start_encoding(item.1.sender))?;
        track!(self.round.start_encoding(item.1.round))?;
        track_assert_eq!(item.1.message_id, None, ErrorKind::InconsistentState);
        Ok(())
    }

    fn requiring_bytes(&self) -> ByteCount {
        ByteCount::Finite(self.exact_requiring_bytes())
    }

    fn is_idle(&self) -> bool {
        self.round.is_idle()
    }
}
impl<M: MessagePayload> SizedEncode for GraftOptimizeMessageEncoder<M> {
    fn exact_requiring_bytes(&self) -> u64 {
        self.destination.exact_requiring_bytes()
            + self.sender.exact_requiring_bytes()
            + self.round.exact_requiring_bytes()
    }
}

#[derive(Debug)]
pub struct PruneMessageDecoder<M> {
    destination: LocalNodeIdDecoder,
    sender: NodeIdDecoder,
    _phantom: PhantomData<M>,
}
impl<M> Default for PruneMessageDecoder<M> {
    fn default() -> Self {
        PruneMessageDecoder {
            destination: Default::default(),
            sender: Default::default(),
            _phantom: PhantomData,
        }
    }
}
impl<M: MessagePayload> Decode for PruneMessageDecoder<M> {
    type Item = (LocalNodeId, PruneMessage<M>);

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        bytecodec_try_decode!(self.destination, offset, buf, eos);
        bytecodec_try_decode!(self.sender, offset, buf, eos);
        Ok(offset)
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        let destination = track!(self.destination.finish_decoding())?;
        let sender = track!(self.sender.finish_decoding())?;

        let message = PruneMessage { sender };
        Ok((destination, message))
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.destination
            .requiring_bytes()
            .add_for_decoding(self.sender.requiring_bytes())
    }

    fn is_idle(&self) -> bool {
        self.sender.is_idle()
    }
}

#[derive(Debug)]
pub struct PruneMessageEncoder<M> {
    destination: LocalNodeIdEncoder,
    sender: NodeIdEncoder,
    _phantom: PhantomData<M>,
}
impl<M> Default for PruneMessageEncoder<M> {
    fn default() -> Self {
        PruneMessageEncoder {
            destination: Default::default(),
            sender: Default::default(),
            _phantom: PhantomData,
        }
    }
}
impl<M: MessagePayload> Encode for PruneMessageEncoder<M> {
    type Item = (LocalNodeId, PruneMessage<M>);

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        bytecodec_try_encode!(self.destination, offset, buf, eos);
        bytecodec_try_encode!(self.sender, offset, buf, eos);
        Ok(offset)
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track!(self.destination.start_encoding(item.0))?;
        track!(self.sender.start_encoding(item.1.sender))?;
        Ok(())
    }

    fn requiring_bytes(&self) -> ByteCount {
        ByteCount::Finite(self.exact_requiring_bytes())
    }

    fn is_idle(&self) -> bool {
        self.sender.is_idle()
    }
}
impl<M: MessagePayload> SizedEncode for PruneMessageEncoder<M> {
    fn exact_requiring_bytes(&self) -> u64 {
        self.destination.exact_requiring_bytes() + self.sender.exact_requiring_bytes()
    }
}
