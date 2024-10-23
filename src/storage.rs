use std::ops::Bound;

use crate::key::{Key, KeyEncodeResult};

pub trait Storage {
    fn get<K: Key>(&self, key: K) -> KeyEncodeResult<Option<Vec<u8>>, K>;
}
pub trait StorageMut: Storage {
    fn set<K: Key>(&mut self, key: K, value: Vec<u8>) -> KeyEncodeResult<(), K>;
    fn delete<K: Key>(&mut self, key: K) -> KeyEncodeResult<(), K>;
}

pub type Iter<'a, T> = Box<dyn Iterator<Item = T> + 'a>;

pub trait IterableStorage: Storage {
    fn keys<K: Key>(&self, low: Bound<K>, high: Bound<K>) -> KeyEncodeResult<Iter<Vec<u8>>, K>;
    fn iter<K: Key>(
        &self,
        low: Bound<K>,
        high: Bound<K>,
    ) -> KeyEncodeResult<Iter<(Vec<u8>, Vec<u8>)>, K>;
}

impl Storage for std::collections::HashMap<Vec<u8>, Vec<u8>> {
    fn get<K: Key>(&self, key: K) -> KeyEncodeResult<Option<Vec<u8>>, K> {
        Ok(Self::get(self, &key.encode()?).cloned())
    }
}

impl StorageMut for std::collections::HashMap<Vec<u8>, Vec<u8>> {
    fn set<K: Key>(&mut self, key: K, value: Vec<u8>) -> KeyEncodeResult<(), K> {
        Self::insert(self, key.encode()?, value);
        Ok(())
    }

    fn delete<K: Key>(&mut self, key: K) -> KeyEncodeResult<(), K> {
        Self::remove(self, &key.encode()?);
        Ok(())
    }
}

impl Storage for std::collections::BTreeMap<Vec<u8>, Vec<u8>> {
    fn get<K: Key>(&self, key: K) -> KeyEncodeResult<Option<Vec<u8>>, K> {
        Ok(Self::get(self, &key.encode()?).cloned())
    }
}

impl StorageMut for std::collections::BTreeMap<Vec<u8>, Vec<u8>> {
    fn set<K: Key>(&mut self, key: K, value: Vec<u8>) -> KeyEncodeResult<(), K> {
        Self::insert(self, key.encode()?, value);
        Ok(())
    }

    fn delete<K: Key>(&mut self, key: K) -> KeyEncodeResult<(), K> {
        Self::remove(self, &key.encode()?);
        Ok(())
    }
}

impl IterableStorage for std::collections::BTreeMap<Vec<u8>, Vec<u8>> {
    fn keys<K: Key>(&self, low: Bound<K>, high: Bound<K>) -> KeyEncodeResult<Iter<Vec<u8>>, K> {
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
        Ok(Box::new(iter.map(|(k, _)| k.clone())))
    }

    fn iter<K: Key>(
        &self,
        low: Bound<K>,
        high: Bound<K>,
    ) -> KeyEncodeResult<Iter<(Vec<u8>, Vec<u8>)>, K> {
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
        Ok(Box::new(iter.map(|(k, v)| (k.clone(), v.clone()))))
    }
}
