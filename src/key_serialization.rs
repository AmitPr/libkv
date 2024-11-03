use crate::{Decodable, Encodable, Encoding};

use super::{KeyDeserializeError, KeySerializeError};

/// Encoding implemented for types that serialize to byte strings
///
/// If the type is [`Ord`] or [`PartialOrd`], it is the implementor's
/// responsibility to ensure that the serialized representation preserves
/// lexicographic ordering guarantees.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyEncoding;
impl Encoding for KeyEncoding {
    type EncodeError = KeySerializeError;
    type DecodeError = KeyDeserializeError;
}

/// Represents a key that can be either a pre-serialized byte sequence or a key of type K.
pub enum KeyType<K> {
    /// A serialized key (of assumed type K).
    Raw(Vec<u8>),
    /// A key of type K, pre-serialization.
    Key(K),
}
impl<K: Encodable<KeyEncoding>> Encodable<KeyEncoding> for KeyType<K> {
    fn encode(&self) -> Result<Vec<u8>, KeySerializeError> {
        match self {
            Self::Raw(key) => Ok(key.clone()),
            Self::Key(key) => key.encode(),
        }
    }
}

impl<K: Decodable<KeyEncoding>> Decodable<KeyEncoding> for KeyType<K> {
    fn decode(bytes: &mut &[u8]) -> Result<Self, KeyDeserializeError> {
        K::decode(bytes).map(Self::Key)
    }
}

impl Encodable<KeyEncoding> for () {
    fn encode(&self) -> Result<Vec<u8>, KeySerializeError> {
        Ok(vec![])
    }
}

impl Decodable<KeyEncoding> for () {
    fn decode(_: &mut &[u8]) -> Result<Self, KeyDeserializeError> {
        Ok(())
    }
}

impl Encodable<KeyEncoding> for String {
    fn encode(&self) -> Result<Vec<u8>, KeySerializeError> {
        let length = encode_length(self.len())?;
        let mut encoded = Vec::with_capacity(self.len() + length.len());
        encoded.extend(length);
        encoded.extend(self.as_bytes());
        Ok(encoded)
    }
}

impl Decodable<KeyEncoding> for String {
    fn decode(bytes: &mut &[u8]) -> Result<Self, KeyDeserializeError> {
        let len = decode_length(bytes)?;
        let (data, rest) = bytes
            .split_at_checked(len)
            .ok_or(KeyDeserializeError::NotEnoughBytes(len, bytes.len()))?;
        *bytes = rest;
        String::from_utf8(data.to_vec()).map_err(Into::into)
    }
}

impl Encodable<KeyEncoding> for std::borrow::Cow<'_, [u8]> {
    fn encode(&self) -> Result<Vec<u8>, KeySerializeError> {
        let length = encode_length(self.len())?;
        let mut encoded = Vec::with_capacity(self.len() + length.len());
        encoded.extend(length);
        encoded.extend(self.iter());
        Ok(encoded)
    }
}

impl Decodable<KeyEncoding> for std::borrow::Cow<'_, [u8]> {
    fn decode(bytes: &mut &[u8]) -> Result<Self, KeyDeserializeError> {
        let len = decode_length(bytes)?;
        let (data, rest) = bytes
            .split_at_checked(len)
            .ok_or(KeyDeserializeError::NotEnoughBytes(len, bytes.len()))?;
        *bytes = rest;
        Ok(std::borrow::Cow::Owned(data.to_vec()))
    }
}

impl Encodable<KeyEncoding> for Vec<u8> {
    fn encode(&self) -> Result<Vec<u8>, KeySerializeError> {
        let length = encode_length(self.len())?;
        let mut encoded = Vec::with_capacity(self.len() + length.len());
        encoded.extend(length);
        encoded.extend(self);
        Ok(encoded)
    }
}

impl Decodable<KeyEncoding> for Vec<u8> {
    fn decode(bytes: &mut &[u8]) -> Result<Self, KeyDeserializeError> {
        let len = decode_length(bytes)?;
        let (data, rest) = bytes
            .split_at_checked(len)
            .ok_or(KeyDeserializeError::NotEnoughBytes(len, bytes.len()))?;
        *bytes = rest;
        Ok(data.to_vec())
    }
}

impl<K: Encodable<KeyEncoding>> Encodable<KeyEncoding> for Option<K> {
    fn encode(&self) -> Result<Vec<u8>, KeySerializeError> {
        match self {
            Some(k) => k.encode(),
            None => Ok(vec![]),
        }
    }
}

impl<K: Decodable<KeyEncoding>> Decodable<KeyEncoding> for Option<K> {
    fn decode(bytes: &mut &[u8]) -> Result<Self, KeyDeserializeError> {
        if bytes.is_empty() {
            return Ok(None);
        }
        K::decode(bytes).map(Some)
    }
}

macro_rules! impl_tuple_keyserde {
    ($l: ident $f: ident, $($v:ident $t:ident),+) => {
        impl_tuple_keyserde!($($v $t),+);
        impl<$f: Encodable<KeyEncoding>, $($t:Encodable<KeyEncoding>,)+> Encodable<KeyEncoding> for ($f, $($t,)+)
        {
            fn encode(&self) -> Result<Vec<u8>, KeySerializeError> {

                let ($l, $($v,)+) = self;
                let mut encoded = Vec::new();
                encoded.extend($l.encode()?);
                $(
                    encoded.extend($v.encode()?);
                )+
                Ok(encoded)
            }
        }
        impl<$f: Decodable<KeyEncoding>, $($t:Decodable<KeyEncoding>,)+> Decodable<KeyEncoding> for ($f, $($t,)+)
        {
            fn decode(bytes: &mut &[u8]) -> Result<Self, KeyDeserializeError> {
                let $l = $f::decode(bytes)?;
                $(
                    let $v = $t::decode(bytes)?;
                )+
                Ok(($l, $($v,)+))
            }
        }
    };

    ($l: ident $f: ident) => {}
}

impl_tuple_keyserde!(t T, u U, v V, w W, x X, y Y, z Z, a A, b B, c C, d D, e E);

macro_rules! impl_uint_keyserde {
    ($($t:ty),+) => {
        $(
            impl Encodable<KeyEncoding> for $t {
                fn encode(&self) -> Result<Vec<u8>, KeySerializeError> {
                    Ok(self.to_be_bytes().to_vec())
                }
            }

            impl Decodable<KeyEncoding> for $t {
                fn decode(bytes: &mut &[u8]) -> Result<Self, KeyDeserializeError> {
                    type Bytes = [u8; std::mem::size_of::<$t>()];
                    let (int_bytes, rest): (&Bytes, &[u8]) =
                        bytes
                            .split_first_chunk()
                            .ok_or(KeyDeserializeError::NotEnoughBytes(
                                std::mem::size_of::<$t>(),
                                bytes.len(),
                            ))?;
                    *bytes = rest;
                    Ok(<$t>::from_be_bytes(*int_bytes))
                }
            }
        )+
    };
}

impl_uint_keyserde!(u8, u16, u32, u64, u128, usize);

macro_rules! impl_sint_keyserde {
    ($($t:ty),+) => {
        $(
            impl Encodable<KeyEncoding> for $t {
                fn encode(&self) -> Result<Vec<u8>, KeySerializeError> {
                    // Map signed to unsigned
                    // x = x ^ $t::MIN
                    let x = self ^ <$t>::MIN;
                    Ok(x.to_be_bytes().to_vec())
                }
            }

            impl Decodable<KeyEncoding> for $t {
                fn decode(bytes: &mut &[u8]) -> Result<Self, KeyDeserializeError> {
                    type Bytes = [u8; std::mem::size_of::<$t>()];
                    let (int_bytes, rest): (&Bytes, &[u8]) =
                        bytes
                            .split_first_chunk()
                            .ok_or(KeyDeserializeError::NotEnoughBytes(
                                std::mem::size_of::<$t>(),
                                bytes.len(),
                            ))?;
                    *bytes = rest;
                    // Map back to signed
                    let x = <$t>::from_be_bytes(*int_bytes) ^ <$t>::MIN;
                    Ok(x)
                }
            }
        )+
    };
}
impl_sint_keyserde!(i8, i16, i32, i64, i128, isize);

/// Encodes `len` as a compact integer with the format:
///
/// [0-4 bits]: length of compact representation, in bytes (0-8), > 8 is invalid
/// [5-7 bits]: 4 high bits of len
/// [8-15 bits]: 8 next bits of len
/// ...
///
/// In other words: a 4-bit length-prefixed variable-length integer encoding.
///
/// This format can encode any unsigned value up to 2^125 - 1 (2^4 * 8 + 4 bits)
/// Serialized, this has the nice property that it obeys lexicographic ordering.
fn encode_length(mut len: usize) -> Result<Vec<u8>, KeySerializeError> {
    if len <= 0xf {
        return Ok(vec![len as u8]);
    }
    // Get position of highest set bit
    let high = len.ilog2() as usize;
    let num_bytes = (high + 4) / 8;
    // debug_assert!(num_bytes <= 0b1111, "usize is too large to encode");
    let mut bytes = vec![0; num_bytes + 1];
    let mut idx = num_bytes;
    while idx > 0 {
        bytes[idx] = (len & 0xFF) as u8;
        len >>= 8;
        idx -= 1;
    }
    bytes[0] = (num_bytes << 4) as u8 | (len & 0x0F) as u8;
    Ok(bytes)
}

/// Decodes an unsigned integer from the format used in [`encode_length`].
fn decode_length(bytes: &mut &[u8]) -> Result<usize, KeyDeserializeError> {
    let first = bytes
        .first()
        .ok_or(KeyDeserializeError::NotEnoughBytes(1, bytes.len()))?;

    let mut num_bytes = *first as usize >> 4;
    if bytes.len() < num_bytes {
        return Err(KeyDeserializeError::NotEnoughBytes(num_bytes, bytes.len()));
    }
    let mut decoded = (*first & 0x0F) as usize;

    let mut idx = 1;
    while num_bytes > 0 {
        decoded <<= 8;
        decoded |= *bytes.get(idx).unwrap() as usize;
        num_bytes -= 1;
        idx += 1;
    }

    *bytes = &bytes[idx..];
    Ok(decoded)
}

#[cfg(test)]
mod test {
    use crate::{
        decode, encode,
        key_serialization::{decode_length, encode_length},
        KeyEncoding,
    };

    #[test]
    fn test_length_serde() {
        let test_cases = vec![
            0,
            1,
            15,
            16,
            255,
            256,
            257,
            1024,
            1025,
            1026,
            4095,
            4096,
            4097,
            65535,
            65536,
            65537,
            0xFFFFFFFF,
            0x100000000,
            0x100000001,
            0xFFFFFFFFFFFFFFFF,
            0x7FFFFFFFFFFFFFFF,
            0x8000000000000000,
            0x8000000000000001,
        ];
        for len in test_cases {
            let encoded = encode_length(len).unwrap();
            let decoded = decode_length(&mut encoded.as_slice()).unwrap();
            assert_eq!(len, decoded);
        }
    }

    #[test]
    fn test_keyserde() {
        let key = 42usize;
        let encoded = encode::<KeyEncoding>(&key).unwrap();
        let decoded = decode::<usize, KeyEncoding>(encoded.as_slice()).unwrap();
        assert_eq!(key, decoded);

        let key = 42u64;
        let encoded = encode::<KeyEncoding>(&key).unwrap();
        let decoded = decode::<u64, KeyEncoding>(encoded.as_slice()).unwrap();
        assert_eq!(key, decoded);

        let key = 42i64;
        let encoded = encode::<KeyEncoding>(&key).unwrap();
        let decoded = decode::<i64, KeyEncoding>(encoded.as_slice()).unwrap();
        assert_eq!(key, decoded);

        let key = -42i64;
        let encoded = encode::<KeyEncoding>(&key).unwrap();
        let decoded = decode::<i64, KeyEncoding>(encoded.as_slice()).unwrap();
        assert_eq!(key, decoded);

        let key = "hello".to_string();
        let encoded = encode::<KeyEncoding>(&key).unwrap();
        let decoded = decode::<String, KeyEncoding>(encoded.as_slice()).unwrap();
        assert_eq!(key, decoded);

        let key = vec![1, 2, 3, 4];
        let encoded = encode::<KeyEncoding>(&key).unwrap();
        let decoded = decode::<Vec<u8>, KeyEncoding>(encoded.as_slice()).unwrap();
        assert_eq!(key, decoded);

        let key = (42usize, "hello".to_string());
        let encoded = encode::<KeyEncoding>(&key).unwrap();
        let decoded = decode::<(usize, String), KeyEncoding>(encoded.as_slice()).unwrap();
        assert_eq!(key, decoded);

        let key = (42usize, "hello".to_string(), vec![1, 2, 3, 4]);
        let encoded = encode::<KeyEncoding>(&key).unwrap();
        let decoded = decode::<(usize, String, Vec<u8>), KeyEncoding>(encoded.as_slice()).unwrap();
        assert_eq!(key, decoded);

        let key = (42usize, "hello".to_string(), vec![1, 2, 3, 4], 42u64);
        let encoded = encode::<KeyEncoding>(&key).unwrap();
        let decoded =
            decode::<(usize, String, Vec<u8>, u64), KeyEncoding>(encoded.as_slice()).unwrap();
        assert_eq!(key, decoded);
    }
}
