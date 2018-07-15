use bytecodec::bytes::{BytesEncoder, CopyableBytesDecoder};
use bytecodec::combinator::Peekable;
use bytecodec::fixnum::{
    U16beDecoder, U16beEncoder, U32beDecoder, U32beEncoder, U8Decoder, U8Encoder,
};
use bytecodec::{ByteCount, Decode, Encode, Eos, ErrorKind, Result, SizedEncode};
use std::net::{SocketAddr, SocketAddrV4, SocketAddrV6};

#[derive(Debug, Default)]
pub struct SocketAddrDecoder {
    version: Peekable<U8Decoder>,
    v4: SocketAddrV4Decoder,
    v6: SocketAddrV6Decoder,
}
impl Decode for SocketAddrDecoder {
    type Item = SocketAddr;

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        bytecodec_try_decode!(self.version, offset, buf, eos);
        match self.version.peek().cloned().expect("Never fails") {
            4 => bytecodec_try_decode!(self.v4, offset, buf, eos),
            6 => bytecodec_try_decode!(self.v6, offset, buf, eos),
            v => track_panic!(ErrorKind::InvalidInput, "Unknown IP version: {}", v),
        }
        Ok(offset)
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        let v = track!(self.version.finish_decoding())?;
        match v {
            4 => track!(self.v4.finish_decoding()).map(SocketAddr::V4),
            6 => track!(self.v6.finish_decoding()).map(SocketAddr::V6),
            _ => track_panic!(ErrorKind::InvalidInput, "Unknown IP version: {}", v),
        }
    }

    fn requiring_bytes(&self) -> ByteCount {
        match self.version.peek().cloned() {
            None => self.version.requiring_bytes(),
            Some(4) => self.v4.requiring_bytes(),
            Some(6) => self.v6.requiring_bytes(),
            Some(_) => ByteCount::Unknown,
        }
    }

    fn is_idle(&self) -> bool {
        match self.version.peek().cloned() {
            Some(4) => self.v4.is_idle(),
            Some(6) => self.v6.is_idle(),
            _ => false,
        }
    }
}

#[derive(Debug, Default)]
struct SocketAddrV4Decoder {
    ip: CopyableBytesDecoder<[u8; 4]>,
    port: U16beDecoder,
}
impl Decode for SocketAddrV4Decoder {
    type Item = SocketAddrV4;

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        bytecodec_try_decode!(self.ip, offset, buf, eos);
        bytecodec_try_decode!(self.port, offset, buf, eos);
        Ok(offset)
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        let ip = track!(self.ip.finish_decoding())?.into();
        let port = track!(self.port.finish_decoding())?;
        Ok(SocketAddrV4::new(ip, port))
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.ip
            .requiring_bytes()
            .add_for_decoding(self.port.requiring_bytes())
    }

    fn is_idle(&self) -> bool {
        self.port.is_idle()
    }
}

#[derive(Debug, Default)]
struct SocketAddrV6Decoder {
    ip: CopyableBytesDecoder<[u8; 16]>,
    port: U16beDecoder,
    flowinfo: U32beDecoder,
    scope_id: U32beDecoder,
}
impl Decode for SocketAddrV6Decoder {
    type Item = SocketAddrV6;

    fn decode(&mut self, buf: &[u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        bytecodec_try_decode!(self.ip, offset, buf, eos);
        bytecodec_try_decode!(self.port, offset, buf, eos);
        bytecodec_try_decode!(self.flowinfo, offset, buf, eos);
        bytecodec_try_decode!(self.scope_id, offset, buf, eos);
        Ok(offset)
    }

    fn finish_decoding(&mut self) -> Result<Self::Item> {
        let ip = track!(self.ip.finish_decoding())?.into();
        let port = track!(self.port.finish_decoding())?;
        let flowinfo = track!(self.flowinfo.finish_decoding())?;
        let scope_id = track!(self.scope_id.finish_decoding())?;
        Ok(SocketAddrV6::new(ip, port, flowinfo, scope_id))
    }

    fn requiring_bytes(&self) -> ByteCount {
        self.ip
            .requiring_bytes()
            .add_for_decoding(self.port.requiring_bytes())
            .add_for_decoding(self.flowinfo.requiring_bytes())
            .add_for_decoding(self.scope_id.requiring_bytes())
    }

    fn is_idle(&self) -> bool {
        self.scope_id.is_idle()
    }
}

#[derive(Debug, Default)]
pub struct SocketAddrEncoder {
    version: U8Encoder,
    v4: SocketAddrV4Encoder,
    v6: SocketAddrV6Encoder,
}
impl Encode for SocketAddrEncoder {
    type Item = SocketAddr;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        bytecodec_try_encode!(self.version, offset, buf, eos);
        bytecodec_try_encode!(self.v4, offset, buf, eos);
        bytecodec_try_encode!(self.v6, offset, buf, eos);
        Ok(offset)
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        match item {
            SocketAddr::V4(x) => {
                track!(self.version.start_encoding(4))?;
                track!(self.v4.start_encoding(x))?;
            }
            SocketAddr::V6(x) => {
                track!(self.version.start_encoding(6))?;
                track!(self.v6.start_encoding(x))?;
            }
        }
        Ok(())
    }

    fn requiring_bytes(&self) -> ByteCount {
        ByteCount::Finite(self.exact_requiring_bytes())
    }

    fn is_idle(&self) -> bool {
        self.version.is_idle() && self.v4.is_idle() && self.v6.is_idle()
    }
}
impl SizedEncode for SocketAddrEncoder {
    fn exact_requiring_bytes(&self) -> u64 {
        self.version.exact_requiring_bytes()
            + self.v4.exact_requiring_bytes()
            + self.v6.exact_requiring_bytes()
    }
}

#[derive(Debug, Default)]
struct SocketAddrV4Encoder {
    ip: BytesEncoder<[u8; 4]>,
    port: U16beEncoder,
}
impl Encode for SocketAddrV4Encoder {
    type Item = SocketAddrV4;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        bytecodec_try_encode!(self.ip, offset, buf, eos);
        bytecodec_try_encode!(self.port, offset, buf, eos);
        Ok(offset)
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track!(self.ip.start_encoding(item.ip().octets()))?;
        track!(self.port.start_encoding(item.port()))?;
        Ok(())
    }

    fn requiring_bytes(&self) -> ByteCount {
        ByteCount::Finite(self.exact_requiring_bytes())
    }

    fn is_idle(&self) -> bool {
        self.ip.is_idle() && self.port.is_idle()
    }
}
impl SizedEncode for SocketAddrV4Encoder {
    fn exact_requiring_bytes(&self) -> u64 {
        self.ip.exact_requiring_bytes() + self.port.exact_requiring_bytes()
    }
}

#[derive(Debug, Default)]
struct SocketAddrV6Encoder {
    ip: BytesEncoder<[u8; 16]>,
    port: U16beEncoder,
    flowinfo: U32beEncoder,
    scope_id: U32beEncoder,
}
impl Encode for SocketAddrV6Encoder {
    type Item = SocketAddrV6;

    fn encode(&mut self, buf: &mut [u8], eos: Eos) -> Result<usize> {
        let mut offset = 0;
        bytecodec_try_encode!(self.ip, offset, buf, eos);
        bytecodec_try_encode!(self.port, offset, buf, eos);
        bytecodec_try_encode!(self.flowinfo, offset, buf, eos);
        bytecodec_try_encode!(self.scope_id, offset, buf, eos);
        Ok(offset)
    }

    fn start_encoding(&mut self, item: Self::Item) -> Result<()> {
        track!(self.ip.start_encoding(item.ip().octets()))?;
        track!(self.port.start_encoding(item.port()))?;
        track!(self.flowinfo.start_encoding(item.flowinfo()))?;
        track!(self.scope_id.start_encoding(item.scope_id()))?;
        Ok(())
    }

    fn requiring_bytes(&self) -> ByteCount {
        ByteCount::Finite(self.exact_requiring_bytes())
    }

    fn is_idle(&self) -> bool {
        self.ip.is_idle()
            && self.port.is_idle()
            && self.flowinfo.is_idle()
            && self.scope_id.is_idle()
    }
}
impl SizedEncode for SocketAddrV6Encoder {
    fn exact_requiring_bytes(&self) -> u64 {
        self.ip.exact_requiring_bytes()
            + self.port.exact_requiring_bytes()
            + self.flowinfo.exact_requiring_bytes()
            + self.scope_id.exact_requiring_bytes()
    }
}
