use bytecodec::fixnum::{U64beDecoder, U64beEncoder};
use bytecodec::{ByteCount, Decode, Encode, Eos, Result, SizedEncode};

use super::net::{SocketAddrDecoder, SocketAddrEncoder};
use node::{LocalNodeId, NodeId};

#[derive(Debug, Default)]
pub struct LocalNodeIdDecoder(U64beDecoder);
impl Decode for LocalNodeIdDecoder {
    type Item = LocalNodeId;

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        track!(self.0.decode(buf, eos))
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        track!(self.0.finish_decoding()).map(LocalNodeId::new)
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.0.requiring_bytes()
    }

    fn is_idle(&self) -> bool {
        self.0.is_idle()
    }
}

#[derive(Debug, Default)]
pub struct NodeIdDecoder {
    addr: SocketAddrDecoder,
    local_id: LocalNodeIdDecoder,
}
impl Decode for NodeIdDecoder {
    type Item = NodeId;

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        bytecodec_try_decode!(self.addr, offset, buf, eos);
        bytecodec_try_decode!(self.local_id, offset, buf, eos);
        Ok(offset)
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        let addr = track!(self.addr.finish_decoding())?;
        let local_id = track!(self.local_id.finish_decoding())?;
        Ok(NodeId::new(addr, local_id))
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.addr
            .requiring_bytes()
            .add_for_decoding(self.local_id.requiring_bytes())
    }

    fn is_idle(&self) -> bool {
        self.addr.is_idle() && self.local_id.is_idle()
    }
}

#[derive(Debug, Default)]
pub struct LocalNodeIdEncoder(U64beEncoder);
impl Encode for LocalNodeIdEncoder {
    type Item = LocalNodeId;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        track!(self.0.encode(buf, eos))
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track!(self.0.start_encoding(item.value()))
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.0.requiring_bytes()
    }

    fn is_idle(&self) -> bool {
        self.0.is_idle()
    }
}
impl SizedEncode for LocalNodeIdEncoder {
    fn exact_requiring_bytes(&self) -> u64 {
        self.0.exact_requiring_bytes()
    }
}

#[derive(Debug, Default)]
pub struct NodeIdEncoder {
    addr: SocketAddrEncoder,
    local_id: LocalNodeIdEncoder,
}
impl Encode for NodeIdEncoder {
    type Item = NodeId;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        bytecodec_try_encode!(self.addr, offset, buf, eos);
        bytecodec_try_encode!(self.local_id, offset, buf, eos);
        Ok(offset)
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track!(self.addr.start_encoding(item.address()))?;
        track!(self.local_id.start_encoding(item.local_id()))?;
        Ok(())
    }

    fn requiring_bytes(&self) -> ByteCount {
        ByteCount::Finite(self.exact_requiring_bytes())
    }

    fn is_idle(&self) -> bool {
        self.addr.is_idle() && self.local_id.is_idle()
    }
}
impl SizedEncode for NodeIdEncoder {
    fn exact_requiring_bytes(&self) -> u64 {
        self.addr.exact_requiring_bytes() + self.local_id.exact_requiring_bytes()
    }
}
