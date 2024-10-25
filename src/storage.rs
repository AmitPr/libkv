use super::{KeySerde, KeySerializeError};
use std::ops::Bound;

pub trait Storage {
    fn get<K: KeySerde>(&self, key: &K) -> Result<Option<Vec<u8>>, KeySerializeError> {
        Ok(self.get_raw(&key.encode()?))
    }

    fn get_raw(&self, key: &[u8]) -> Option<Vec<u8>>;
}
pub trait StorageMut: Storage {
    fn set<K: KeySerde>(&mut self, key: &K, value: Vec<u8>) -> Result<(), KeySerializeError> {
        self.set_raw(key.encode()?, value);
        Ok(())
    }
    fn set_raw(&mut self, key: Vec<u8>, value: Vec<u8>);

    fn delete<K: KeySerde>(&mut self, key: &K) -> Result<(), KeySerializeError> {
        self.delete_raw(&key.encode()?);
        Ok(())
    }
    fn delete_raw(&mut self, key: &[u8]);
}

pub type Iter<'a, T> = Box<dyn Iterator<Item = T> + 'a>;

pub trait IterableStorage: Storage {
    fn keys<K: KeySerde>(
        &self,
        low: Bound<K>,
        high: Bound<K>,
    ) -> Result<Iter<Vec<u8>>, KeySerializeError>;

    #[allow(clippy::type_complexity)]
    fn iter<K: KeySerde>(
        &self,
        low: Bound<K>,
        high: Bound<K>,
    ) -> Result<Iter<(Vec<u8>, Vec<u8>)>, KeySerializeError>;
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

impl IterableStorage for std::collections::BTreeMap<Vec<u8>, Vec<u8>> {
    fn keys<K: KeySerde>(
        &self,
        low: Bound<K>,
        high: Bound<K>,
    ) -> Result<Iter<Vec<u8>>, KeySerializeError> {
        let low = encode_bound!(low);
        let high = encode_bound!(high);
        let iter = self.range((low, high));
        Ok(Box::new(iter.map(|(k, _)| k.clone())))
    }

    fn iter<K: KeySerde>(
        &self,
        low: Bound<K>,
        high: Bound<K>,
    ) -> Result<Iter<(Vec<u8>, Vec<u8>)>, KeySerializeError> {
        let low = encode_bound!(low);
        let high = encode_bound!(high);
        let iter = self.range((low, high));
        Ok(Box::new(iter.map(|(k, v)| (k.clone(), v.clone()))))
    }
}
