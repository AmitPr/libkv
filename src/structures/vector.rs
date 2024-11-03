#![allow(unused)]

use std::ops::Bound;

use crate::{
    DataStructure, DsIter, Encodable, Item, IterableStorage, KeyEncoding, KeySerializeError, Map,
    NonTerminal, Order, Storage, StorageError,
};

// TODO: Vector needs some sort of pre-save/delete hook to update the counter.

pub struct Vector<'a, V: DataStructure> {
    map: Map<'a, usize, V>,
    /// Counter serialized using usize Key encoding.
    counter: Item<'a, usize, KeyEncoding>,
}

impl<'a, V: DataStructure> DataStructure for Vector<'a, V> {
    type Key = (usize, Option<V::Key>);
    type DsType = NonTerminal;
    type Enc = V::Enc;
    type Value = V::Value;

    fn with_prefix(prefix: Vec<u8>) -> Self {
        Self {
            map: Map::with_prefix(prefix.clone()),
            counter: Item::with_prefix(prefix),
        }
    }

    fn should_skip_key(key: &Self::Key) -> bool {
        // Vector has keys
        //      prefix -> counter
        //      prefix/0 -> value
        //      ...
        //      prefix/n -> value
        // We skip the counter key, which is when the second element is None.
        key.1.as_ref().map_or(true, V::should_skip_key)
    }
}

impl<V: DataStructure> Vector<'static, V> {
    pub const fn new(key: &'static [u8]) -> Self {
        Self {
            map: Map::new(key),
            counter: Item::new(key),
        }
    }
}

impl<'a, V: DataStructure> Vector<'a, V> {
    fn key(&self, key: &usize) -> Result<Vec<u8>, KeySerializeError> {
        let encoded = Encodable::<KeyEncoding>::encode(key)?;
        let full = [self.map.prefix(), &encoded].concat();

        Ok(full)
    }

    pub fn len<S: Storage>(&self, storage: &S) -> Result<usize, StorageError<KeyEncoding>> {
        self.counter.may_load(storage).map(|v| v.unwrap_or(0))
    }

    pub fn at(&self, index: usize) -> Result<V, KeySerializeError> {
        Ok(V::with_prefix(self.key(&index)?))
    }

    pub fn range<'b, S: IterableStorage>(
        &self,
        storage: &'b S,
        start: Bound<usize>,
        end: Bound<usize>,
        order: Order,
    ) -> Result<DsIter<'b, Self>, KeySerializeError> {
        let start = match start {
            Bound::Included(k) => Bound::Included(self.key(&k)?),
            Bound::Excluded(k) => Bound::Excluded(self.key(&k)?),
            Bound::Unbounded => Bound::Unbounded,
        };
        let end = match end {
            Bound::Included(k) => Bound::Included(self.key(&k)?),
            Bound::Excluded(k) => Bound::Excluded(self.key(&k)?),
            Bound::Unbounded => Bound::Unbounded,
        };
        let iter = storage.iter(start, end, order)?;
        Ok(DsIter::new(self.map.prefix().to_vec(), iter))
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use crate::mock::DisplayEncoding;

    use super::*;

    #[test]
    fn test_vector() {
        const VECTOR: Vector<Item<String, DisplayEncoding>> = Vector::new(b"foo");

        let mut storage: HashMap<Vec<u8>, Vec<u8>> = HashMap::new();
        assert_eq!(VECTOR.len(&storage), Ok(0));

        VECTOR
            .at(0)
            .unwrap()
            .save(&mut storage, &"bar".to_string())
            .unwrap();

        assert_eq!(
            VECTOR.at(0).unwrap().may_load(&storage),
            Ok(Some("bar".to_string()))
        );
        // TODO: Vector push doesn't increment the counter.
        assert_eq!(VECTOR.len(&storage), Ok(1));
    }
}
