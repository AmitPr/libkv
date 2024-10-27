use std::marker::PhantomData;

use crate::{Codec, Decodable, Encoding, Iter, KeySerde, StorageError};

pub trait DataStructure {
    type Key: KeySerde;
    type Enc: Encoding;
    type Value: Codec<Self::Enc>;
    type DsType: sealed::ContainerType;
    fn with_prefix(prefix: Vec<u8>) -> Self
    where
        Self: Sized;

    fn should_skip_key(key: &Self::Key) -> bool;
}

pub trait Container: DataStructure<DsType = NonTerminal> {
    type Inner: DataStructure;
}

mod sealed {
    pub trait ContainerType: Sized {}
}

pub struct Terminal {}
impl sealed::ContainerType for Terminal {}
pub struct NonTerminal {}
impl sealed::ContainerType for NonTerminal {}

pub struct DsIter<'a, D: DataStructure> {
    _marker: PhantomData<D>,
    prefix: Vec<u8>,
    iter: Iter<'a, (Vec<u8>, Vec<u8>)>,
}

impl<'a, D: DataStructure> DsIter<'a, D> {
    pub const fn new(prefix: Vec<u8>, iter: Iter<'a, (Vec<u8>, Vec<u8>)>) -> Self {
        Self {
            _marker: PhantomData,
            prefix,
            iter,
        }
    }
}

impl<'a, D: DataStructure> Iterator for DsIter<'a, D> {
    type Item = Result<(D::Key, D::Value), StorageError<D::Enc>>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (mut key_bytes, val_bytes) = self.iter.next()?;
            if !key_bytes.starts_with(&self.prefix) {
                return None;
            }
            let key_bytes = key_bytes.split_off(self.prefix.len());
            let key = <D as DataStructure>::Key::decode(&mut key_bytes.as_slice())
                .map(|k| (D::should_skip_key(&k), k));

            match key {
                Ok((true, _)) => continue,
                Ok((false, key)) => {
                    let val = <D::Value as Decodable<D::Enc>>::decode(&val_bytes)
                        .map_err(StorageError::ValueDeserialize);
                    return Some(val.map(|val| (key, val)));
                }
                Err(e) => return Some(Err(StorageError::KeyDeserialize(e))),
            }
        }
    }
}
