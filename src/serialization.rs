use std::error::Error;

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
