use std::{marker::PhantomData, ops::Bound};

use super::{
    container::Container,
    key::{EncodeResult, Key},
};

pub trait Storage {
    fn get<K: Key>(&self, key: K) -> EncodeResult<Option<Vec<u8>>, K> {
        Ok(self.get_raw(&key.encode()?))
    }

    fn get_raw(&self, key: &[u8]) -> Option<Vec<u8>>;
}
pub trait StorageMut: Storage {
    fn set<K: Key>(&mut self, key: K, value: Vec<u8>) -> EncodeResult<(), K> {
        self.set_raw(key.encode()?, value);
        Ok(())
    }
    fn set_raw(&mut self, key: Vec<u8>, value: Vec<u8>);

    fn delete<K: Key>(&mut self, key: K) -> EncodeResult<(), K> {
        self.delete_raw(&key.encode()?);
        Ok(())
    }
    fn delete_raw(&mut self, key: &[u8]);
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
    fn get_raw(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.get(key).cloned()
    }
}

impl StorageMut for std::collections::HashMap<Vec<u8>, Vec<u8>> {
    fn set_raw(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.insert(key, value);
    }

    fn delete_raw(&mut self, key: &[u8]) {
        self.remove(key);
    }
}

impl Storage for std::collections::BTreeMap<Vec<u8>, Vec<u8>> {
    fn get_raw(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.get(key).cloned()
    }
}

impl StorageMut for std::collections::BTreeMap<Vec<u8>, Vec<u8>> {
    fn set_raw(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.insert(key, value);
    }

    fn delete_raw(&mut self, key: &[u8]) {
        self.remove(key);
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
