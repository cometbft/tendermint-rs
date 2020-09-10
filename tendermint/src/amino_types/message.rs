use prost::encoding::encoded_len_varint;
use std::convert::TryInto;

/// Extend the original prost::Message trait with a few helper functions in order to
/// reduce boiler-plate code (and without modifying the prost dependency).
pub trait AminoMessage: prost::Message {
    /// Directly encode a prost message into a freshly created Vec<u8>.
    /// This can be useful when passing those bytes directly to a hasher, or,
    /// to reduce boiler plate code when working with the encoded bytes.
    ///
    /// Warning: Only use this method, if you are in control what will be encoded.
    /// If there is an encoding error, this method will panic.
    fn bytes_vec(&self) -> Vec<u8>
    where
        Self: Sized,
    {
        let mut res = Vec::with_capacity(self.encoded_len());
        self.encode(&mut res).unwrap();
        res
    }

    /// Encode prost message as length delimited.
    ///
    /// Warning: Only use this method, if you are in control what will be encoded.
    /// If there is an encoding error, this method will panic.
    fn bytes_vec_length_delimited(&self) -> Vec<u8>
    where
        Self: Sized,
    {
        let len = self.encoded_len();
        let mut res =
            Vec::with_capacity(len + encoded_len_varint(len.try_into().expect("length overflow")));
        self.encode_length_delimited(&mut res).unwrap();
        res
    }
}
impl<M: prost::Message> AminoMessage for M {
    // blanket impl
}
