//! Encoding/decoding mechanisms for ABCI requests and responses.
//!
//! Implements the [Tendermint Socket Protocol][tsp].
//!
//! [tsp]: https://github.com/tendermint/tendermint/blob/v0.34.x/spec/abci/client-server.md#tsp

use std::{
    io::{Read, Write},
    marker::PhantomData,
};

use bytes::{Buf, BufMut, BytesMut};
use prost::Message;
use tendermint_proto::v0_38::abci::{Request, Response};

use crate::error::Error;

/// The maximum number of bytes we expect in a varint. We use this to check if
/// we're encountering a decoding error for a varint.
pub const MAX_VARINT_LENGTH: usize = 16;

/// The maximum length of a single message, matching CometBFT's `maxMsgSize`.
pub const MAX_MESSAGE_LENGTH: u64 = i32::MAX as u64;

/// The server receives incoming requests, and sends outgoing responses.
pub type ServerCodec<S> = Codec<S, Request, Response>;

#[cfg(feature = "client")]
/// The client sends outgoing requests, and receives incoming responses.
pub type ClientCodec<S> = Codec<S, Response, Request>;

/// Allows for iteration over `S` to produce instances of `I`, as well as
/// sending instances of `O`.
pub struct Codec<S, I, O> {
    stream: S,
    // Long-running read buffer
    read_buf: BytesMut,
    // Fixed-length read window
    read_window: Vec<u8>,
    write_buf: BytesMut,
    _incoming: PhantomData<I>,
    _outgoing: PhantomData<O>,
}

impl<S, I, O> Codec<S, I, O>
where
    S: Read + Write,
    I: Message + Default,
    O: Message,
{
    /// Constructor.
    pub fn new(stream: S, read_buf_size: usize) -> Self {
        Self {
            stream,
            read_buf: BytesMut::new(),
            read_window: vec![0_u8; read_buf_size],
            write_buf: BytesMut::new(),
            _incoming: Default::default(),
            _outgoing: Default::default(),
        }
    }
}

// Iterating over a codec produces instances of `Result<I>`.
impl<S, I, O> Iterator for Codec<S, I, O>
where
    S: Read,
    I: Message + Default,
{
    type Item = Result<I, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // Try to decode an incoming message from our buffer first
            match decode_length_delimited::<I>(&mut self.read_buf) {
                Ok(Some(incoming)) => return Some(Ok(incoming)),
                Err(e) => return Some(Err(e)),
                _ => (), // not enough data to decode a message, let's continue.
            }

            // If we don't have enough data to decode a message, try to read
            // more
            let bytes_read = match self.stream.read(self.read_window.as_mut()) {
                Ok(br) => br,
                Err(e) => return Some(Err(Error::io(e))),
            };
            if bytes_read == 0 {
                // The underlying stream terminated
                return None;
            }
            self.read_buf
                .extend_from_slice(&self.read_window[..bytes_read]);
        }
    }
}

impl<S, I, O> Codec<S, I, O>
where
    S: Write,
    O: Message,
{
    /// Send a message using this codec.
    pub fn send(&mut self, message: O) -> Result<(), Error> {
        encode_length_delimited(message, &mut self.write_buf)?;
        while !self.write_buf.is_empty() {
            let bytes_written = self
                .stream
                .write(self.write_buf.as_ref())
                .map_err(Error::io)?;

            if bytes_written == 0 {
                return Err(Error::io(std::io::Error::new(
                    std::io::ErrorKind::WriteZero,
                    "failed to write to underlying stream",
                )));
            }
            self.write_buf.advance(bytes_written);
        }

        self.stream.flush().map_err(Error::io)?;

        Ok(())
    }
}

/// Encode the given message with a length prefix.
pub fn encode_length_delimited<M, B>(message: M, mut dst: &mut B) -> Result<(), Error>
where
    M: Message,
    B: BufMut,
{
    let mut buf = BytesMut::new();
    message.encode(&mut buf).map_err(Error::encode)?;

    let buf = buf.freeze();
    prost::encoding::encode_varint(buf.len() as u64, &mut dst);
    dst.put(buf);
    Ok(())
}

/// Attempt to decode a message of type `M` from the given source buffer.
pub fn decode_length_delimited<M>(src: &mut BytesMut) -> Result<Option<M>, Error>
where
    M: Message + Default,
{
    let mut tmp = &src[..];
    let encoded_len = match prost::encoding::decode_varint(&mut tmp) {
        Ok(len) => len,
        // We've potentially only received a partial length delimiter
        Err(_) if src.len() <= MAX_VARINT_LENGTH => return Ok(None),
        Err(e) => return Err(Error::decode(e)),
    };
    if encoded_len > MAX_MESSAGE_LENGTH {
        return Err(Error::io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("message length {encoded_len} exceeds maximum {MAX_MESSAGE_LENGTH}"),
        )));
    }
    if (tmp.len() as u64) < encoded_len {
        // We don't have enough data yet to decode the entire message
        return Ok(None);
    }
    src.advance(src.len() - tmp.len());
    let res = M::decode(src.split_to(encoded_len as usize)).map_err(Error::decode)?;
    Ok(Some(res))
}

#[cfg(test)]
mod tests {
    use tendermint_proto::v0_38::abci::{request, RequestEcho};

    use super::*;

    fn echo(message: &str) -> Request {
        Request {
            value: Some(request::Value::Echo(RequestEcho {
                message: message.to_owned(),
            })),
        }
    }

    #[test]
    fn rejects_oversized_length_prefix() {
        let mut buf = BytesMut::new();
        prost::encoding::encode_varint(i32::MAX as u64 + 1, &mut buf);
        assert!(decode_length_delimited::<Request>(&mut buf).is_err());
    }

    #[test]
    fn waits_for_max_sized_message() {
        let mut buf = BytesMut::new();
        prost::encoding::encode_varint(i32::MAX as u64, &mut buf);
        assert!(decode_length_delimited::<Request>(&mut buf)
            .unwrap()
            .is_none());
    }

    #[test]
    fn decodes_consecutive_messages_and_partial_tail() {
        let mut buf = BytesMut::new();
        encode_length_delimited(echo("a"), &mut buf).unwrap();
        encode_length_delimited(echo("b"), &mut buf).unwrap();
        let mut partial = BytesMut::new();
        encode_length_delimited(echo("c"), &mut partial).unwrap();
        buf.extend_from_slice(&partial[..partial.len() - 1]);

        let first = decode_length_delimited::<Request>(&mut buf).unwrap();
        let second = decode_length_delimited::<Request>(&mut buf).unwrap();
        assert_eq!(first, Some(echo("a")));
        assert_eq!(second, Some(echo("b")));
        assert_eq!(decode_length_delimited::<Request>(&mut buf).unwrap(), None);
        assert_eq!(buf.len(), partial.len() - 1);
    }
}
