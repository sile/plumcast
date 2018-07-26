use bytecodec::combinator::{Collect, Repeat};
use bytecodec::fixnum::{U8Decoder, U8Encoder};
use bytecodec::{ByteCount, Decode, Encode, Eos, Result, SizedEncode};
use hyparview::TimeToLive;
use std;

use super::node::{LocalNodeIdDecoder, LocalNodeIdEncoder, NodeIdDecoder, NodeIdEncoder};
use hyparview_misc::{
    DisconnectMessage, ForwardJoinMessage, JoinMessage, NeighborMessage, ShuffleMessage,
    ShuffleReplyMessage,
};
use {LocalNodeId, NodeId};

#[derive(Debug, Default)]
pub struct JoinMessageDecoder {
    destination: LocalNodeIdDecoder,
    sender: NodeIdDecoder,
}
impl Decode for JoinMessageDecoder {
    type Item = (LocalNodeId, JoinMessage);

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        bytecodec_try_decode!(self.destination, offset, buf, eos);
        bytecodec_try_decode!(self.sender, offset, buf, eos);
        Ok(offset)
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        let destination = track!(self.destination.finish_decoding())?;
        let sender = track!(self.sender.finish_decoding())?;
        Ok((destination, JoinMessage { sender }))
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
pub struct JoinMessageEncoder {
    destination: LocalNodeIdEncoder,
    sender: NodeIdEncoder,
}
impl Encode for JoinMessageEncoder {
    type Item = (LocalNodeId, JoinMessage);

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
impl SizedEncode for JoinMessageEncoder {
    fn exact_requiring_bytes(&self) -> u64 {
        self.destination.exact_requiring_bytes() + self.sender.exact_requiring_bytes()
    }
}

#[derive(Debug, Default)]
pub struct ForwardJoinMessageDecoder {
    destination: LocalNodeIdDecoder,
    sender: NodeIdDecoder,
    new_node: NodeIdDecoder,
    ttl: U8Decoder,
}
impl Decode for ForwardJoinMessageDecoder {
    type Item = (LocalNodeId, ForwardJoinMessage);

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        bytecodec_try_decode!(self.destination, offset, buf, eos);
        bytecodec_try_decode!(self.sender, offset, buf, eos);
        bytecodec_try_decode!(self.new_node, offset, buf, eos);
        bytecodec_try_decode!(self.ttl, offset, buf, eos);
        Ok(offset)
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        let destination = track!(self.destination.finish_decoding())?;
        let sender = track!(self.sender.finish_decoding())?;
        let new_node = track!(self.new_node.finish_decoding())?;
        let ttl = track!(self.ttl.finish_decoding())?;
        let message = ForwardJoinMessage {
            sender,
            new_node,
            ttl: TimeToLive::new(ttl),
        };
        Ok((destination, message))
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.destination
            .requiring_bytes()
            .add_for_decoding(self.sender.requiring_bytes())
            .add_for_decoding(self.new_node.requiring_bytes())
            .add_for_decoding(self.ttl.requiring_bytes())
    }

    fn is_idle(&self) -> bool {
        self.ttl.is_idle()
    }
}

#[derive(Debug, Default)]
pub struct ForwardJoinMessageEncoder {
    destination: LocalNodeIdEncoder,
    sender: NodeIdEncoder,
    new_node: NodeIdEncoder,
    ttl: U8Encoder,
}
impl Encode for ForwardJoinMessageEncoder {
    type Item = (LocalNodeId, ForwardJoinMessage);

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        bytecodec_try_encode!(self.destination, offset, buf, eos);
        bytecodec_try_encode!(self.sender, offset, buf, eos);
        bytecodec_try_encode!(self.new_node, offset, buf, eos);
        bytecodec_try_encode!(self.ttl, offset, buf, eos);
        Ok(offset)
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track!(self.destination.start_encoding(item.0))?;
        track!(self.sender.start_encoding(item.1.sender))?;
        track!(self.new_node.start_encoding(item.1.new_node))?;
        track!(self.ttl.start_encoding(item.1.ttl.as_u8()))?;
        Ok(())
    }

    fn requiring_bytes(&self) -> ByteCount {
        ByteCount::Finite(self.exact_requiring_bytes())
    }

    fn is_idle(&self) -> bool {
        self.ttl.is_idle()
    }
}
impl SizedEncode for ForwardJoinMessageEncoder {
    fn exact_requiring_bytes(&self) -> u64 {
        self.destination.exact_requiring_bytes()
            + self.sender.exact_requiring_bytes()
            + self.new_node.exact_requiring_bytes()
            + self.ttl.exact_requiring_bytes()
    }
}

#[derive(Debug, Default)]
pub struct NeighborMessageDecoder {
    destination: LocalNodeIdDecoder,
    sender: NodeIdDecoder,
    high_priority: U8Decoder,
}
impl Decode for NeighborMessageDecoder {
    type Item = (LocalNodeId, NeighborMessage);

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        bytecodec_try_decode!(self.destination, offset, buf, eos);
        bytecodec_try_decode!(self.sender, offset, buf, eos);
        bytecodec_try_decode!(self.high_priority, offset, buf, eos);
        Ok(offset)
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        let destination = track!(self.destination.finish_decoding())?;
        let sender = track!(self.sender.finish_decoding())?;
        let high_priority = track!(self.high_priority.finish_decoding())? != 0;
        let message = NeighborMessage {
            sender,
            high_priority,
        };
        Ok((destination, message))
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.destination
            .requiring_bytes()
            .add_for_decoding(self.sender.requiring_bytes())
            .add_for_decoding(self.high_priority.requiring_bytes())
    }

    fn is_idle(&self) -> bool {
        self.high_priority.is_idle()
    }
}

#[derive(Debug, Default)]
pub struct NeighborMessageEncoder {
    destination: LocalNodeIdEncoder,
    sender: NodeIdEncoder,
    high_priority: U8Encoder,
}
impl Encode for NeighborMessageEncoder {
    type Item = (LocalNodeId, NeighborMessage);

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        bytecodec_try_encode!(self.destination, offset, buf, eos);
        bytecodec_try_encode!(self.sender, offset, buf, eos);
        bytecodec_try_encode!(self.high_priority, offset, buf, eos);
        Ok(offset)
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track!(self.destination.start_encoding(item.0))?;
        track!(self.sender.start_encoding(item.1.sender))?;
        track!(
            self.high_priority
                .start_encoding(item.1.high_priority as u8)
        )?;
        Ok(())
    }

    fn requiring_bytes(&self) -> ByteCount {
        ByteCount::Finite(self.exact_requiring_bytes())
    }

    fn is_idle(&self) -> bool {
        self.high_priority.is_idle()
    }
}
impl SizedEncode for NeighborMessageEncoder {
    fn exact_requiring_bytes(&self) -> u64 {
        self.destination.exact_requiring_bytes()
            + self.sender.exact_requiring_bytes()
            + self.high_priority.exact_requiring_bytes()
    }
}

#[derive(Debug, Default)]
pub struct ShuffleMessageDecoder {
    destination: LocalNodeIdDecoder,
    sender: NodeIdDecoder,
    origin: NodeIdDecoder,
    ttl: U8Decoder,
    nodes: Collect<NodeIdDecoder, Vec<NodeId>>,
}
impl Decode for ShuffleMessageDecoder {
    type Item = (LocalNodeId, ShuffleMessage);

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        bytecodec_try_decode!(self.destination, offset, buf, eos);
        bytecodec_try_decode!(self.sender, offset, buf, eos);
        bytecodec_try_decode!(self.origin, offset, buf, eos);
        bytecodec_try_decode!(self.ttl, offset, buf, eos);
        bytecodec_try_decode!(self.nodes, offset, buf, eos);
        Ok(offset)
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        let destination = track!(self.destination.finish_decoding())?;
        let sender = track!(self.sender.finish_decoding())?;
        let origin = track!(self.origin.finish_decoding())?;
        let ttl = track!(self.ttl.finish_decoding())?;
        let nodes = track!(self.nodes.finish_decoding())?;

        let message = ShuffleMessage {
            sender,
            origin,
            ttl: TimeToLive::new(ttl),
            nodes,
        };
        Ok((destination, message))
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.destination
            .requiring_bytes()
            .add_for_decoding(self.sender.requiring_bytes())
            .add_for_decoding(self.origin.requiring_bytes())
            .add_for_decoding(self.ttl.requiring_bytes())
            .add_for_decoding(self.nodes.requiring_bytes())
    }

    fn is_idle(&self) -> bool {
        self.nodes.is_idle()
    }
}

#[derive(Debug, Default)]
pub struct ShuffleMessageEncoder {
    destination: LocalNodeIdEncoder,
    sender: NodeIdEncoder,
    origin: NodeIdEncoder,
    ttl: U8Encoder,
    nodes: Repeat<NodeIdEncoder, std::vec::IntoIter<NodeId>>,
}
impl Encode for ShuffleMessageEncoder {
    type Item = (LocalNodeId, ShuffleMessage);

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        bytecodec_try_encode!(self.destination, offset, buf, eos);
        bytecodec_try_encode!(self.sender, offset, buf, eos);
        bytecodec_try_encode!(self.origin, offset, buf, eos);
        bytecodec_try_encode!(self.ttl, offset, buf, eos);
        bytecodec_try_encode!(self.nodes, offset, buf, eos);
        Ok(offset)
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track!(self.destination.start_encoding(item.0))?;
        track!(self.sender.start_encoding(item.1.sender))?;
        track!(self.origin.start_encoding(item.1.origin))?;
        track!(self.ttl.start_encoding(item.1.ttl.as_u8()))?;
        track!(self.nodes.start_encoding(item.1.nodes.into_iter()))?;
        Ok(())
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.destination
            .requiring_bytes()
            .add_for_encoding(self.sender.requiring_bytes())
            .add_for_encoding(self.origin.requiring_bytes())
            .add_for_encoding(self.ttl.requiring_bytes())
            .add_for_encoding(self.nodes.requiring_bytes())
    }

    fn is_idle(&self) -> bool {
        self.nodes.is_idle()
    }
}

#[derive(Debug, Default)]
pub struct ShuffleReplyMessageDecoder {
    destination: LocalNodeIdDecoder,
    sender: NodeIdDecoder,
    nodes: Collect<NodeIdDecoder, Vec<NodeId>>,
}
impl Decode for ShuffleReplyMessageDecoder {
    type Item = (LocalNodeId, ShuffleReplyMessage);

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        bytecodec_try_decode!(self.destination, offset, buf, eos);
        bytecodec_try_decode!(self.sender, offset, buf, eos);
        bytecodec_try_decode!(self.nodes, offset, buf, eos);
        Ok(offset)
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        let destination = track!(self.destination.finish_decoding())?;
        let sender = track!(self.sender.finish_decoding())?;
        let nodes = track!(self.nodes.finish_decoding())?;

        let message = ShuffleReplyMessage { sender, nodes };
        Ok((destination, message))
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.destination
            .requiring_bytes()
            .add_for_decoding(self.sender.requiring_bytes())
            .add_for_decoding(self.nodes.requiring_bytes())
    }

    fn is_idle(&self) -> bool {
        self.nodes.is_idle()
    }
}

#[derive(Debug, Default)]
pub struct ShuffleReplyMessageEncoder {
    destination: LocalNodeIdEncoder,
    sender: NodeIdEncoder,
    nodes: Repeat<NodeIdEncoder, std::vec::IntoIter<NodeId>>,
}
impl Encode for ShuffleReplyMessageEncoder {
    type Item = (LocalNodeId, ShuffleReplyMessage);

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        bytecodec_try_encode!(self.destination, offset, buf, eos);
        bytecodec_try_encode!(self.sender, offset, buf, eos);
        bytecodec_try_encode!(self.nodes, offset, buf, eos);
        Ok(offset)
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track!(self.destination.start_encoding(item.0))?;
        track!(self.sender.start_encoding(item.1.sender))?;
        track!(self.nodes.start_encoding(item.1.nodes.into_iter()))?;
        Ok(())
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.destination
            .requiring_bytes()
            .add_for_encoding(self.sender.requiring_bytes())
            .add_for_encoding(self.nodes.requiring_bytes())
    }

    fn is_idle(&self) -> bool {
        self.nodes.is_idle()
    }
}

#[derive(Debug, Default)]
pub struct DisconnectMessageDecoder {
    destination: LocalNodeIdDecoder,
    sender: NodeIdDecoder,
    alive: U8Decoder,
}
impl Decode for DisconnectMessageDecoder {
    type Item = (LocalNodeId, DisconnectMessage);

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        bytecodec_try_decode!(self.destination, offset, buf, eos);
        bytecodec_try_decode!(self.sender, offset, buf, eos);
        bytecodec_try_decode!(self.alive, offset, buf, eos);
        Ok(offset)
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        let destination = track!(self.destination.finish_decoding())?;
        let sender = track!(self.sender.finish_decoding())?;
        let alive = track!(self.alive.finish_decoding())? != 0;
        Ok((destination, DisconnectMessage { sender, alive }))
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.destination
            .requiring_bytes()
            .add_for_decoding(self.sender.requiring_bytes())
            .add_for_decoding(self.alive.requiring_bytes())
    }

    fn is_idle(&self) -> bool {
        self.alive.is_idle()
    }
}

#[derive(Debug, Default)]
pub struct DisconnectMessageEncoder {
    destination: LocalNodeIdEncoder,
    sender: NodeIdEncoder,
    alive: U8Encoder,
}
impl Encode for DisconnectMessageEncoder {
    type Item = (LocalNodeId, DisconnectMessage);

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        bytecodec_try_encode!(self.destination, offset, buf, eos);
        bytecodec_try_encode!(self.sender, offset, buf, eos);
        bytecodec_try_encode!(self.alive, offset, buf, eos);
        Ok(offset)
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track!(self.destination.start_encoding(item.0))?;
        track!(self.sender.start_encoding(item.1.sender))?;
        track!(self.alive.start_encoding(item.1.alive as u8))?;
        Ok(())
    }

    fn requiring_bytes(&self) -> ByteCount {
        ByteCount::Finite(self.exact_requiring_bytes())
    }

    fn is_idle(&self) -> bool {
        self.alive.is_idle()
    }
}
impl SizedEncode for DisconnectMessageEncoder {
    fn exact_requiring_bytes(&self) -> u64 {
        self.destination.exact_requiring_bytes()
            + self.sender.exact_requiring_bytes()
            + self.alive.exact_requiring_bytes()
    }
}
