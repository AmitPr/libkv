use crate::{Encodable, KeyEncoding};

use super::KeySerializeError;
use std::ops::Bound;

pub enum Order {
    Ascending,
    Descending,
}

pub trait Storage {
    fn get<K: Encodable<KeyEncoding>>(
        &self,
        key: &K,
    ) -> Result<Option<Vec<u8>>, KeySerializeError> {
        Ok(self.get_raw(&key.encode()?))
    }

    fn get_raw(&self, key: &[u8]) -> Option<Vec<u8>>;
}
pub trait StorageMut: Storage {
    fn set<K: Encodable<KeyEncoding>>(
        &mut self,
        key: &K,
        value: Vec<u8>,
    ) -> Result<(), KeySerializeError> {
        self.set_raw(key.encode()?, value);
        Ok(())
    }
    fn set_raw(&mut self, key: Vec<u8>, value: Vec<u8>);

    fn delete<K: Encodable<KeyEncoding>>(&mut self, key: &K) -> Result<(), KeySerializeError> {
        self.delete_raw(&key.encode()?);
        Ok(())
    }
    fn delete_raw(&mut self, key: &[u8]);
}

pub type Iter<'a, T> = Box<dyn Iterator<Item = T> + 'a>;

pub trait IterableStorage: Storage {
    fn keys<K: Encodable<KeyEncoding>>(
        &self,
        low: Bound<K>,
        high: Bound<K>,
        order: Order,
    ) -> Result<Iter<Vec<u8>>, KeySerializeError>;

    #[allow(clippy::type_complexity)]
    fn iter<K: Encodable<KeyEncoding>>(
        &self,
        low: Bound<K>,
        high: Bound<K>,
        order: Order,
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

fn is_empty_range(start: &Bound<Vec<u8>>, end: &Bound<Vec<u8>>) -> bool {
    match (start, end) {
        // If one bound is Included, then start must be strictly greater than end
        // for the range to be empty.
        (Bound::Included(start), Bound::Included(end))
        | (Bound::Included(start), Bound::Excluded(end))
        | (Bound::Excluded(start), Bound::Included(end)) => start > end,

        // If both bounds are Excluded, then start must be greater than or equal to end
        // for the range to be empty.
        (Bound::Excluded(start), Bound::Excluded(end)) => start >= end,
        // If either bound is Unbounded, then the range is not empty.
        _ => false,
    }
}

fn clone_kv((k, v): (&Vec<u8>, &Vec<u8>)) -> (Vec<u8>, Vec<u8>) {
    (k.clone(), v.clone())
}

fn clone_k((k, _): (&Vec<u8>, &Vec<u8>)) -> Vec<u8> {
    k.clone()
}

impl IterableStorage for std::collections::BTreeMap<Vec<u8>, Vec<u8>> {
    fn keys<K: Encodable<KeyEncoding>>(
        &self,
        low: Bound<K>,
        high: Bound<K>,
        order: Order,
    ) -> Result<Iter<Vec<u8>>, KeySerializeError> {
        let low = encode_bound!(low);
        let high = encode_bound!(high);

        // BTreeMap::range panics if low > high or low == high, with Bound::Excluded
        if is_empty_range(&low, &high) {
            return Ok(Box::new(std::iter::empty()));
        }

        let iter = self.range((low, high));
        match order {
            Order::Ascending => Ok(Box::new(iter.map(clone_k))),
            Order::Descending => Ok(Box::new(iter.rev().map(clone_k))),
        }
    }

    fn iter<K: Encodable<KeyEncoding>>(
        &self,
        low: Bound<K>,
        high: Bound<K>,
        order: Order,
    ) -> Result<Iter<(Vec<u8>, Vec<u8>)>, KeySerializeError> {
        let low = encode_bound!(low);
        let high = encode_bound!(high);
        // BTreeMap::range panics if low > high or low == high, with Bound::Excluded
        if is_empty_range(&low, &high) {
            return Ok(Box::new(std::iter::empty()));
        }

        let iter = self.range((low, high));
        match order {
            Order::Ascending => Ok(Box::new(iter.map(clone_kv))),
            Order::Descending => Ok(Box::new(iter.rev().map(clone_kv))),
        }
    }
}
