use std::error::Error;

pub trait Encoding {
    type EncodeError: Error;
    type DecodeError: Error;
}

pub trait Encodable<E: Encoding> {
    /// Try to encode this value to bytes.
    fn encode(&self) -> Result<Vec<u8>, E::EncodeError>;
}

pub trait Decodable<E: Encoding>: Sized {
    /// Try to decode this value from a byte slice, advancing the slice to reflect the number of
    /// bytes consumed.
    fn decode(bytes: &mut &[u8]) -> Result<Self, E::DecodeError>;
}

/// Sugar for implementing both `Encodable` and `Decodable` for a given encoding
/// on a type.
pub trait Codec<E: Encoding>: Encodable<E> + Decodable<E> {}
impl<T: Encodable<E> + Decodable<E>, E: Encoding> Codec<E> for T {}

/// Encode a value using the given encoding, sugar/helper for [`Encodable::encode`]
pub fn encode<E: Encoding>(value: &impl Encodable<E>) -> Result<Vec<u8>, E::EncodeError> {
    value.encode()
}

/// Decode a value using the given encoding, sugar/helper for [`Decodable::decode`]
pub fn decode<T: Decodable<E>, E: Encoding>(bytes: &[u8]) -> Result<T, E::DecodeError> {
    let mut bytes = bytes;
    T::decode(&mut bytes)
}

// /// Encoding that allows any type implementing `serde::Serialize` and `serde::Deserialize` to be
// /// used as a codec.
// #[cfg(feature = "serde")]
// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub struct SerdeEncoding<'de, S: serde::Serializer, D: serde::Deserializer<'de>>(
//     std::marker::PhantomData<(S, &'de D)>,
// );

// #[cfg(feature = "serde")]
// impl<'de, S: serde::Serializer, D: serde::Deserializer<'de>> Encoding for SerdeEncoding<'de, S, D> {
//     type EncodeError = S::Error;
//     type DecodeError = D::Error;
// }

// #[cfg(feature = "serde")]
// impl<'de, T, S: serde::Serializer, D: serde::Deserializer<'de>> Encodable<SerdeEncoding<'de, S, D>>
//     for T
// where
//     T: serde::Serialize,
// {
//     fn encode(&self) -> Result<Vec<u8>, S::Error> {
//         // serde_json::to_string uses 128 bytes as the default capacity
//         let mut buf = Vec::with_capacity(128);
//         S::serialize_
//         S::serialize
//     }
// }

#[cfg(feature = "borsh")]
pub(crate) mod _borsh {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct BorshEncoding;

    impl Encoding for BorshEncoding {
        type EncodeError = ::borsh::io::Error;
        type DecodeError = ::borsh::io::Error;
    }

    impl<T: ::borsh::ser::BorshSerialize> Encodable<BorshEncoding> for T {
        fn encode(&self) -> Result<Vec<u8>, ::borsh::io::Error> {
            ::borsh::to_vec(self)
        }
    }

    impl<T: ::borsh::de::BorshDeserialize> Decodable<BorshEncoding> for T {
        fn decode(bytes: &[u8]) -> Result<Self, ::borsh::io::Error> {
            ::borsh::from_slice(bytes)
        }
    }
}
