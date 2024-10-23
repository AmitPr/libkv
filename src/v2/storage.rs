use std::{marker::PhantomData, ops::Bound};

use super::{
    container::Container,
    key::{EncodeResult, Key},
};

pub trait Storage {
    fn get<K: Key>(&self, key: K) -> EncodeResult<Option<Vec<u8>>, K>;
}
pub trait StorageMut: Storage {
    fn set<K: Key>(&mut self, key: K, value: Vec<u8>) -> EncodeResult<(), K>;
    fn delete<K: Key>(&mut self, key: K) -> EncodeResult<(), K>;
}

pub type Iter<'a, T> = Box<dyn Iterator<Item = T> + 'a>;
pub struct RawKey<C: Container + ?Sized>(pub(crate) Vec<u8>, PhantomData<C>);

pub trait IterableStorage: Storage {
    fn keys<K: Key>(
        &self,
        low: Bound<K>,
        high: Bound<K>,
    ) -> EncodeResult<Iter<RawKey<K::Container>>, K>;
    fn iter<K: Key>(
        &self,
        low: Bound<K>,
        high: Bound<K>,
    ) -> EncodeResult<Iter<(RawKey<K::Container>, Vec<u8>)>, K>;
}

impl Storage for std::collections::HashMap<Vec<u8>, Vec<u8>> {
    fn get<K: Key>(&self, key: K) -> EncodeResult<Option<Vec<u8>>, K> {
        Ok(Self::get(self, &key.encode()?).cloned())
    }
}

impl StorageMut for std::collections::HashMap<Vec<u8>, Vec<u8>> {
    fn set<K: Key>(&mut self, key: K, value: Vec<u8>) -> EncodeResult<(), K> {
        Self::insert(self, key.encode()?, value);
        Ok(())
    }

    fn delete<K: Key>(&mut self, key: K) -> EncodeResult<(), K> {
        Self::remove(self, &key.encode()?);
        Ok(())
    }
}

impl Storage for std::collections::BTreeMap<Vec<u8>, Vec<u8>> {
    fn get<K: Key>(&self, key: K) -> EncodeResult<Option<Vec<u8>>, K> {
        Ok(Self::get(self, &key.encode()?).cloned())
    }
}

impl StorageMut for std::collections::BTreeMap<Vec<u8>, Vec<u8>> {
    fn set<K: Key>(&mut self, key: K, value: Vec<u8>) -> EncodeResult<(), K> {
        Self::insert(self, key.encode()?, value);
        Ok(())
    }

    fn delete<K: Key>(&mut self, key: K) -> EncodeResult<(), K> {
        Self::remove(self, &key.encode()?);
        Ok(())
    }
}

macro_rules! encode_bound {
    ($bound:expr) => {
        match $bound {
            Bound::Included(k) => Bound::Included(k.encode()?),
            Bound::Excluded(k) => Bound::Excluded(k.encode()?),
            Bound::Unbounded => Bound::Unbounded,
        }
    };
}

pub(super) use encode_bound;

impl IterableStorage for std::collections::BTreeMap<Vec<u8>, Vec<u8>> {
    fn keys<K: Key>(
        &self,
        low: Bound<K>,
        high: Bound<K>,
    ) -> EncodeResult<Iter<RawKey<K::Container>>, K> {
        let low = encode_bound!(low);
        let high = encode_bound!(high);
        let iter = self.range((low, high));
        Ok(Box::new(iter.map(|(k, _)| RawKey(k.clone(), PhantomData))))
    }

    fn iter<K: Key>(
        &self,
        low: Bound<K>,
        high: Bound<K>,
    ) -> EncodeResult<Iter<(RawKey<K::Container>, Vec<u8>)>, K> {
        let low = match low {
            Bound::Included(k) => Bound::Included(k.encode()?),
            Bound::Excluded(k) => Bound::Excluded(k.encode()?),
            Bound::Unbounded => Bound::Unbounded,
        };
        let high = match high {
            Bound::Included(k) => Bound::Included(k.encode()?),
            Bound::Excluded(k) => Bound::Excluded(k.encode()?),
            Bound::Unbounded => Bound::Unbounded,
        };
        let iter = self.range((low, high));
        Ok(Box::new(
            iter.map(|(k, v)| (RawKey(k.clone(), PhantomData), v.clone())),
        ))
    }
}
