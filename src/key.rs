use std::marker::PhantomData;

use crate::Node;

pub type KeyEncodeResult<T, K> = Result<T, <K as KeySerde>::EncodeError>;
pub type KeyDecodeResult<T, K> = Result<T, <K as KeySerde>::DecodeError>;

pub trait KeySerde: Sized {
    type EncodeError;
    type DecodeError;
    fn encode(&self) -> KeyEncodeResult<Vec<u8>, Self>;
    fn decode(bytes: &mut &[u8]) -> KeyDecodeResult<Self, Self>;
}

pub trait Key: KeySerde {
    type Node: Node;
}

#[derive(Debug)]
pub struct CompoundKey<T: Key, U: Key, N: Node>(T, U, PhantomData<N>);

impl<K: Key, N: Node> CompoundKey<K, (), N> {
    pub fn new_prefix(key: K) -> Self {
        CompoundKey(key, (), PhantomData)
    }
}

impl<T: Key, U: Key, N: Node> CompoundKey<T, U, N> {
    pub fn new(prefix: T, suffix: U) -> Self {
        CompoundKey(prefix, suffix, PhantomData)
    }

    pub fn prefix(&self) -> &T {
        &self.0
    }

    pub fn suffix(&self) -> &U {
        &self.1
    }

    pub fn into_inner(self) -> (T, U) {
        (self.0, self.1)
    }
}

impl<T: Key, U: Key, N: Node> KeySerde for CompoundKey<T, U, N> {
    type EncodeError = (Option<T::EncodeError>, Option<U::EncodeError>);
    type DecodeError = (Option<T::DecodeError>, Option<U::DecodeError>);
    fn encode(&self) -> KeyEncodeResult<Vec<u8>, Self> {
        let mut bytes = self.0.encode().map_err(|e| (Some(e), None))?;
        bytes.extend_from_slice(&self.1.encode().map_err(|e| (None, Some(e)))?);
        Ok(bytes)
    }

    fn decode(bytes: &mut &[u8]) -> KeyDecodeResult<Self, Self> {
        let key = T::decode(bytes).map_err(|e| (Some(e), None))?;
        let unit = U::decode(bytes).map_err(|e| (None, Some(e)))?;
        Ok(CompoundKey(key, unit, PhantomData))
    }
}

impl<T: Key, U: Key, N: Node> Key for CompoundKey<T, U, N> {
    type Node = N;
}
