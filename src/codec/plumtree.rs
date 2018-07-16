use bytecodec::bytes::{BytesDecoder, BytesEncoder};
use bytecodec::combinator::Peekable;
use bytecodec::fixnum::{
    U16beDecoder, U16beEncoder, U32beDecoder, U32beEncoder, U64beDecoder, U64beEncoder, U8Decoder,
    U8Encoder,
};
use bytecodec::{ByteCount, Decode, Encode, Eos, Result, SizedEncode};
use plumtree::message::{GossipMessage, GraftMessage, IhaveMessage, Message, PruneMessage};

use super::node::{LocalNodeIdDecoder, LocalNodeIdEncoder, NodeIdDecoder, NodeIdEncoder};
use node::{MessageId, System};
use LocalNodeId;

#[derive(Debug, Default)]
pub struct GossipMessageDecoder {
    destination: LocalNodeIdDecoder,
    sender: NodeIdDecoder,
    message: MessageDecoder,
    round: U16beDecoder,
}
impl Decode for GossipMessageDecoder {
    type Item = (LocalNodeId, GossipMessage<System>);

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        bytecodec_try_decode!(self.destination, offset, buf, eos);
        bytecodec_try_decode!(self.sender, offset, buf, eos);
        bytecodec_try_decode!(self.message, offset, buf, eos);
        bytecodec_try_decode!(self.round, offset, buf, eos);
        Ok(offset)
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        let destination = track!(self.destination.finish_decoding())?;
        let sender = track!(self.sender.finish_decoding())?;
        let message = track!(self.message.finish_decoding())?;
        let round = track!(self.round.finish_decoding())?;
        Ok((
            destination,
            GossipMessage {
                sender,
                message,
                round,
            },
        ))
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.destination
            .requiring_bytes()
            .add_for_decoding(self.sender.requiring_bytes())
            .add_for_decoding(self.message.requiring_bytes())
            .add_for_decoding(self.round.requiring_bytes())
    }

    fn is_idle(&self) -> bool {
        self.round.is_idle()
    }
}

#[derive(Debug, Default)]
struct MessageDecoder {
    id: MessageIdDecoder,
    payload: MessagePayloadDecoder,
}
impl Decode for MessageDecoder {
    type Item = Message<System>;

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
        let node_id = track!(self.node.finish_decoding())?;
        let seqno = track!(self.seqno.finish_decoding())?;
        Ok(MessageId { node_id, seqno })
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

#[derive(Debug, Default)]
pub struct GossipMessageEncoder {
    destination: LocalNodeIdEncoder,
    sender: NodeIdEncoder,
    message: MessageEncoder,
    round: U16beEncoder,
}
impl Encode for GossipMessageEncoder {
    type Item = (LocalNodeId, GossipMessage<System>);

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        bytecodec_try_encode!(self.destination, offset, buf, eos);
        bytecodec_try_encode!(self.sender, offset, buf, eos);
        bytecodec_try_encode!(self.message, offset, buf, eos);
        bytecodec_try_encode!(self.round, offset, buf, eos);
        Ok(offset)
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track!(self.destination.start_encoding(item.0))?;
        track!(self.sender.start_encoding(item.1.sender))?;
        track!(self.message.start_encoding(item.1.message))?;
        track!(self.round.start_encoding(item.1.round))?;
        Ok(())
    }

    fn requiring_bytes(&self) -> ByteCount {
        ByteCount::Finite(self.exact_requiring_bytes())
    }

    fn is_idle(&self) -> bool {
        self.round.is_idle()
    }
}
impl SizedEncode for GossipMessageEncoder {
    fn exact_requiring_bytes(&self) -> u64 {
        self.destination.exact_requiring_bytes()
            + self.sender.exact_requiring_bytes()
            + self.message.exact_requiring_bytes()
            + self.round.exact_requiring_bytes()
    }
}

#[derive(Debug, Default)]
struct MessageEncoder {
    id: MessageIdEncoder,
    payload: MessagePayloadEncoder,
}
impl Encode for MessageEncoder {
    type Item = Message<System>;

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
impl SizedEncode for MessageEncoder {
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
        track!(self.node.start_encoding(item.node_id))?;
        track!(self.seqno.start_encoding(item.seqno))?;
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

#[derive(Debug, Default)]
pub struct IhaveMessageDecoder {
    destination: LocalNodeIdDecoder,
    sender: NodeIdDecoder,
    message_id: MessageIdDecoder,
    round: U16beDecoder,
    realtime: U8Decoder,
}
impl Decode for IhaveMessageDecoder {
    type Item = (LocalNodeId, IhaveMessage<System>);

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        bytecodec_try_decode!(self.destination, offset, buf, eos);
        bytecodec_try_decode!(self.sender, offset, buf, eos);
        bytecodec_try_decode!(self.message_id, offset, buf, eos);
        bytecodec_try_decode!(self.round, offset, buf, eos);
        bytecodec_try_decode!(self.realtime, offset, buf, eos);
        Ok(offset)
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        let destination = track!(self.destination.finish_decoding())?;
        let sender = track!(self.sender.finish_decoding())?;
        let message_id = track!(self.message_id.finish_decoding())?;
        let round = track!(self.round.finish_decoding())?;
        let realtime = track!(self.realtime.finish_decoding())?;

        let message = IhaveMessage {
            sender,
            message_id,
            round,
            realtime: realtime != 0,
        };

        Ok((destination, message))
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.destination
            .requiring_bytes()
            .add_for_decoding(self.sender.requiring_bytes())
            .add_for_decoding(self.message_id.requiring_bytes())
            .add_for_decoding(self.round.requiring_bytes())
            .add_for_decoding(self.realtime.requiring_bytes())
    }

    fn is_idle(&self) -> bool {
        self.realtime.is_idle()
    }
}

#[derive(Debug, Default)]
pub struct IhaveMessageEncoder {
    destination: LocalNodeIdEncoder,
    sender: NodeIdEncoder,
    message_id: MessageIdEncoder,
    round: U16beEncoder,
    realtime: U8Encoder,
}
impl Encode for IhaveMessageEncoder {
    type Item = (LocalNodeId, IhaveMessage<System>);

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        bytecodec_try_encode!(self.destination, offset, buf, eos);
        bytecodec_try_encode!(self.sender, offset, buf, eos);
        bytecodec_try_encode!(self.message_id, offset, buf, eos);
        bytecodec_try_encode!(self.round, offset, buf, eos);
        bytecodec_try_encode!(self.realtime, offset, buf, eos);
        Ok(offset)
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track!(self.destination.start_encoding(item.0))?;
        track!(self.sender.start_encoding(item.1.sender))?;
        track!(self.message_id.start_encoding(item.1.message_id))?;
        track!(self.round.start_encoding(item.1.round))?;
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
impl SizedEncode for IhaveMessageEncoder {
    fn exact_requiring_bytes(&self) -> u64 {
        self.destination.exact_requiring_bytes()
            + self.sender.exact_requiring_bytes()
            + self.message_id.exact_requiring_bytes()
            + self.round.exact_requiring_bytes()
            + self.realtime.exact_requiring_bytes()
    }
}

#[derive(Debug, Default)]
pub struct GraftMessageDecoder {
    destination: LocalNodeIdDecoder,
    sender: NodeIdDecoder,
    has_message_id: Peekable<U8Decoder>,
    message_id: MessageIdDecoder,
    round: U16beDecoder,
}
impl Decode for GraftMessageDecoder {
    type Item = (LocalNodeId, GraftMessage<System>);

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        bytecodec_try_decode!(self.destination, offset, buf, eos);
        bytecodec_try_decode!(self.sender, offset, buf, eos);
        bytecodec_try_decode!(self.has_message_id, offset, buf, eos);
        if self.has_message_id.peek().cloned() == Some(1) {
            bytecodec_try_decode!(self.message_id, offset, buf, eos);
        }
        bytecodec_try_decode!(self.round, offset, buf, eos);
        Ok(offset)
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        let destination = track!(self.destination.finish_decoding())?;
        let sender = track!(self.sender.finish_decoding())?;
        let has_message_id = track!(self.has_message_id.finish_decoding())? == 1;
        let message_id = if has_message_id {
            Some(track!(self.message_id.finish_decoding())?)
        } else {
            None
        };
        let round = track!(self.round.finish_decoding())?;

        let message = GraftMessage {
            sender,
            message_id,
            round,
        };
        Ok((destination, message))
    }

    fn requiring_bytes(&self) -> ByteCount {
        let n = self.destination
            .requiring_bytes()
            .add_for_decoding(self.sender.requiring_bytes())
            .add_for_decoding(self.has_message_id.requiring_bytes())
            .add_for_decoding(self.round.requiring_bytes());
        if self.has_message_id.peek().cloned() == Some(1) {
            n.add_for_decoding(self.message_id.requiring_bytes())
        } else {
            n
        }
    }

    fn is_idle(&self) -> bool {
        self.round.is_idle()
    }
}

#[derive(Debug, Default)]
pub struct GraftMessageEncoder {
    destination: LocalNodeIdEncoder,
    sender: NodeIdEncoder,
    has_message_id: U8Encoder,
    message_id: MessageIdEncoder,
    round: U16beEncoder,
}
impl Encode for GraftMessageEncoder {
    type Item = (LocalNodeId, GraftMessage<System>);

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        bytecodec_try_encode!(self.destination, offset, buf, eos);
        bytecodec_try_encode!(self.sender, offset, buf, eos);
        bytecodec_try_encode!(self.has_message_id, offset, buf, eos);
        bytecodec_try_encode!(self.message_id, offset, buf, eos);
        bytecodec_try_encode!(self.round, offset, buf, eos);
        Ok(offset)
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track!(self.destination.start_encoding(item.0))?;
        track!(self.sender.start_encoding(item.1.sender))?;
        if let Some(message_id) = item.1.message_id {
            track!(self.has_message_id.start_encoding(1))?;
            track!(self.message_id.start_encoding(message_id))?;
        } else {
            track!(self.has_message_id.start_encoding(0))?;
        }
        track!(self.round.start_encoding(item.1.round))?;
        Ok(())
    }

    fn requiring_bytes(&self) -> ByteCount {
        ByteCount::Finite(self.exact_requiring_bytes())
    }

    fn is_idle(&self) -> bool {
        self.round.is_idle()
    }
}
impl SizedEncode for GraftMessageEncoder {
    fn exact_requiring_bytes(&self) -> u64 {
        self.destination.exact_requiring_bytes()
            + self.sender.exact_requiring_bytes()
            + self.has_message_id.exact_requiring_bytes()
            + self.message_id.exact_requiring_bytes()
            + self.round.exact_requiring_bytes()
    }
}

#[derive(Debug, Default)]
pub struct PruneMessageDecoder {
    destination: LocalNodeIdDecoder,
    sender: NodeIdDecoder,
}
impl Decode for PruneMessageDecoder {
    type Item = (LocalNodeId, PruneMessage<System>);

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

#[derive(Debug, Default)]
pub struct PruneMessageEncoder {
    destination: LocalNodeIdEncoder,
    sender: NodeIdEncoder,
}
impl Encode for PruneMessageEncoder {
    type Item = (LocalNodeId, PruneMessage<System>);

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
impl SizedEncode for PruneMessageEncoder {
    fn exact_requiring_bytes(&self) -> u64 {
        self.destination.exact_requiring_bytes() + self.sender.exact_requiring_bytes()
    }
}
