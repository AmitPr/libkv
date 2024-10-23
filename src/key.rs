use std::{convert::Infallible, marker::PhantomData};

use super::containers::traits::Container;

pub type EncodeResult<T, K> = Result<T, <K as KeySerde>::EncodeError>;
pub type DecodeResult<T, K> = Result<T, <K as KeySerde>::DecodeError>;

pub trait KeySerde: Sized {
    type EncodeError;
    type DecodeError;
    fn encode(&self) -> EncodeResult<Vec<u8>, Self>;
    fn decode(bytes: &mut &[u8]) -> Result<Self, Self::DecodeError>;

    type PartialKey: KeySerde;
    fn partial_decode(bytes: &mut &[u8]) -> Result<Option<Self::PartialKey>, Self::DecodeError>;
}

pub trait Key: KeySerde {
    type Container: Container;
}

pub trait AsInner<K> {
    fn as_inner_key(&self) -> Option<&K>;
}

pub struct KeySegment<T: KeySerde, C: Container>(T, PhantomData<C>);
impl<T: KeySerde, C: Container> KeySerde for KeySegment<T, C> {
    type EncodeError = T::EncodeError;
    type DecodeError = T::DecodeError;
    fn encode(&self) -> Result<Vec<u8>, Self::EncodeError> {
        self.0.encode()
    }

    fn decode(bytes: &mut &[u8]) -> Result<Self, Self::DecodeError> {
        T::decode(bytes).map(|t| Self(t, PhantomData))
    }

    type PartialKey = KeySegment<T::PartialKey, C>;
    fn partial_decode(bytes: &mut &[u8]) -> Result<Option<Self::PartialKey>, Self::DecodeError> {
        let t = T::partial_decode(bytes)?;
        let seg = t.map(|t| KeySegment(t, PhantomData));
        Ok(seg)
    }
}

// impl

// impl<T: KeySerde, U: KeySerde, C: Container> From<KeySegment<T, C>> for KeySegment<U, C>
// where
//     U: From<T>,
// {
//     fn from(value: KeySegment<T, C>) -> Self {
//         value.0.into()
//     }
// }

impl<T: KeySerde, C: Container> Key for KeySegment<T, C> {
    type Container = C;
}
impl<T: KeySerde, C: Container> From<T> for KeySegment<T, C> {
    fn from(value: T) -> Self {
        Self(value, PhantomData)
    }
}
pub struct CompoundKey<T, U, C: Container>(T, U, PhantomData<C>);
impl<T: KeySerde, U: KeySerde, C: Container> CompoundKey<T, U, C> {
    pub fn new(t: T, u: U) -> Self {
        Self(t, u, PhantomData)
    }
}

impl<T: KeySerde, U: KeySerde, C: Container> KeySerde for CompoundKey<T, U, C> {
    type EncodeError = (Option<T::EncodeError>, Option<U::EncodeError>);
    type DecodeError = (Option<T::DecodeError>, Option<U::DecodeError>);
    fn encode(&self) -> Result<Vec<u8>, Self::EncodeError> {
        let mut bytes = self.0.encode().map_err(|e| (Some(e), None))?;
        bytes.extend_from_slice(&self.1.encode().map_err(|e| (None, Some(e)))?);
        Ok(bytes)
    }

    fn decode(bytes: &mut &[u8]) -> Result<Self, Self::DecodeError> {
        let t = T::decode(bytes).map_err(|e| (Some(e), None))?;
        let u = U::decode(bytes).map_err(|e| (None, Some(e)))?;
        Ok(Self(t, u, PhantomData))
    }

    type PartialKey = CompoundKey<T::PartialKey, Option<U::PartialKey>, C>;
    fn partial_decode(bytes: &mut &[u8]) -> Result<Option<Self::PartialKey>, Self::DecodeError> {
        let t = T::partial_decode(bytes).map_err(|e| (Some(e), None))?;
        if t.is_none() {
            return Ok(None);
        }
        let t = t.unwrap();
        let u = U::partial_decode(bytes).map_err(|e| (None, Some(e)))?;
        Ok(Some(CompoundKey(t, u, PhantomData)))
    }
}

impl<T: KeySerde, U: KeySerde, C: Container> Key for CompoundKey<T, U, C> {
    type Container = C;
}

impl<T: KeySerde, U: Key, C: Container> AsInner<U> for CompoundKey<T, Option<U>, C> {
    fn as_inner_key(&self) -> Option<&U> {
        self.1.as_ref()
    }
}

#[derive(Debug)]
pub enum KeyDeserializeError {
    NotEnoughBytes,
    DecodeError,
}

impl KeySerde for usize {
    type EncodeError = Infallible;
    type DecodeError = KeyDeserializeError;

    fn encode(&self) -> Result<Vec<u8>, Self::EncodeError> {
        Ok(self.to_be_bytes().to_vec())
    }

    fn decode(bytes: &mut &[u8]) -> Result<Self, Self::DecodeError> {
        let (int_bytes, rest): (&[u8; std::mem::size_of::<usize>()], &[u8]) = bytes
            .split_first_chunk()
            .ok_or(KeyDeserializeError::NotEnoughBytes)?;
        *bytes = rest;

        Ok(usize::from_be_bytes(*int_bytes))
    }

    type PartialKey = Self;
    fn partial_decode(bytes: &mut &[u8]) -> Result<Option<Self::PartialKey>, Self::DecodeError> {
        // Only valid partial if there are either >8 bytes or 0 bytes.
        if bytes.is_empty() {
            return Ok(None);
        }
        let (int_bytes, rest): (&[u8; std::mem::size_of::<usize>()], &[u8]) = bytes
            .split_first_chunk()
            .ok_or(KeyDeserializeError::NotEnoughBytes)?;
        *bytes = rest;

        Ok(Some(usize::from_be_bytes(*int_bytes)))
    }
}

impl KeySerde for () {
    type EncodeError = Infallible;
    type DecodeError = Infallible;

    fn encode(&self) -> Result<Vec<u8>, Self::EncodeError> {
        Ok(vec![])
    }

    fn decode(bytes: &mut &[u8]) -> Result<Self, Self::DecodeError> {
        Ok(())
    }

    type PartialKey = Self;
    fn partial_decode(bytes: &mut &[u8]) -> Result<Option<Self::PartialKey>, Self::DecodeError> {
        Ok(Some(()))
    }
}

impl KeySerde for String {
    type EncodeError = Infallible;
    type DecodeError = KeyDeserializeError;

    fn encode(&self) -> Result<Vec<u8>, Self::EncodeError> {
        let mut encoded = Vec::with_capacity(self.len() + std::mem::size_of::<usize>());
        encoded.extend(self.len().encode()?);
        encoded.extend(self.as_bytes());
        Ok(encoded)
    }

    fn decode(bytes: &mut &[u8]) -> Result<Self, Self::DecodeError> {
        let len = usize::decode(bytes)?;
        let (data, rest) = bytes
            .split_at_checked(len)
            .ok_or(KeyDeserializeError::NotEnoughBytes)?;
        *bytes = rest;
        String::from_utf8(data.to_vec()).map_err(|_| KeyDeserializeError::DecodeError)
    }

    type PartialKey = Self;
    fn partial_decode(bytes: &mut &[u8]) -> Result<Option<Self::PartialKey>, Self::DecodeError> {
        // Only valid if we have no more bytes, or we have enough bytes to decode
        // len + data.
        let len = usize::partial_decode(bytes)?;
        if let Some(len) = len {
            let (data, rest) = bytes
                .split_at_checked(len)
                .ok_or(KeyDeserializeError::NotEnoughBytes)?;
            *bytes = rest;
            Ok(Some(
                String::from_utf8(data.to_vec()).map_err(|_| KeyDeserializeError::DecodeError)?,
            ))
        } else {
            Ok(None)
        }
    }
}

impl<K: KeySerde> KeySerde for Option<K> {
    type EncodeError = K::EncodeError;
    type DecodeError = K::DecodeError;

    fn encode(&self) -> Result<Vec<u8>, Self::EncodeError> {
        match self {
            Some(k) => k.encode(),
            None => Ok(vec![]),
        }
    }

    fn decode(bytes: &mut &[u8]) -> Result<Self, Self::DecodeError> {
        if bytes.is_empty() {
            return Ok(None);
        }
        K::decode(bytes).map(Some)
    }

    type PartialKey = K::PartialKey;
    fn partial_decode(bytes: &mut &[u8]) -> Result<Option<Self::PartialKey>, Self::DecodeError> {
        K::partial_decode(bytes)
    }
}

// macro_rules! compound_key_type {
//     ($t:ident, $($r: ident),+) => {
//         CompoundKey<$t, compound_key_type!($($r),+)>
//     };
//     ($t:ident) => {
//         $t
//     }
// }

// macro_rules! compound_key_into {
//     ($f: tt, $($t:tt),+) => {
//         CompoundKey::new($f, compound_key_into!($($t),+))
//     };
//     ($f: tt) => {
//         $f
//     }
// }

// macro_rules! impl_compound_key_from_tuple {
//     ($l: ident $f: ident, $($v:ident $t:ident),+) => {
//         impl_compound_key_from_tuple!($($v $t),+);
//         impl<$f: Key, $($t:Key,)+> From<($f, $($t,)+)> for compound_key_type!($f, $($t),+)
//         {
//             fn from(($l, $($v,)+): ($f, $($t,)+)) -> Self {
//                 compound_key_into!($l, $($v),+)
//             }
//         }
//     };

//     ($l: ident $f: ident) => {}
// }

// impl_compound_key_from_tuple!(t T, u U, v V, w W, x X, y Y, z Z, a A, b B, c C, d D, e E);
