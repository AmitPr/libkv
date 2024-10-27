use std::error::Error;

use crate::{KeyDeserializeError, KeySerde, KeySerializeError};

pub trait Encoding {
    type EncodeError: Error;
    type DecodeError: Error;
}

pub trait Encodable<E: Encoding> {
    fn encode(&self) -> Result<Vec<u8>, E::EncodeError>;
}

pub trait Decodable<E: Encoding>: Sized {
    fn decode(bytes: &[u8]) -> Result<Self, E::DecodeError>;
}

/// Sugar for implementing both `Encodable` and `Decodable` for a given encoding
/// on a type.
pub trait Codec<E: Encoding>: Encodable<E> + Decodable<E> {}
impl<T: Encodable<E> + Decodable<E>, E: Encoding> Codec<E> for T {}

/// Encoding that simply uses the KeySerde trait for serialization and deserialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyEncoding;
impl Encoding for KeyEncoding {
    type EncodeError = KeySerializeError;
    type DecodeError = KeyDeserializeError;
}

impl<T: KeySerde> Encodable<KeyEncoding> for T {
    fn encode(&self) -> Result<Vec<u8>, KeySerializeError> {
        self.encode()
    }
}

impl<T: KeySerde> Decodable<KeyEncoding> for T {
    fn decode(mut bytes: &[u8]) -> Result<Self, KeyDeserializeError> {
        T::decode(&mut bytes)
    }
}
