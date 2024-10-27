use std::{borrow::Cow, marker::PhantomData, ops::Bound};

use crate::{
    DataStructure, DsIter, IterableStorage, KeySerde, KeySerializeError, NonTerminal, Order,
};

pub struct Map<'a, K: KeySerde, V: DataStructure> {
    prefix: Cow<'a, [u8]>,
    _marker: PhantomData<(K, V)>,
}
impl<'a, K: KeySerde, V: DataStructure> DataStructure for Map<'a, K, V> {
    type Key = (K, Option<V::Key>);
    type DsType = NonTerminal;
    type Enc = V::Enc;
    type Value = V::Value;

    fn with_prefix(prefix: Vec<u8>) -> Self {
        Self {
            prefix: Cow::Owned(prefix),
            _marker: PhantomData,
        }
    }

    fn should_skip_key(key: &Self::Key) -> bool {
        key.1.as_ref().map_or(false, V::should_skip_key)
    }
}

impl<K: KeySerde, V: DataStructure> Map<'static, K, V> {
    pub const fn new(key: &'static [u8]) -> Self {
        Self {
            prefix: Cow::Borrowed(key),
            _marker: PhantomData,
        }
    }
}

impl<'a, K: KeySerde, V: DataStructure> Map<'a, K, V> {
    fn key(&self, key: &K) -> Result<Vec<u8>, KeySerializeError> {
        let encoded = key.encode()?;
        let full = [self.prefix.as_ref(), &encoded].concat();

        Ok(full)
    }

    pub fn prefix(&self) -> &[u8] {
        self.prefix.as_ref()
    }

    pub fn at(&self, key: impl Into<K>) -> Result<V, KeySerializeError> {
        Ok(V::with_prefix(self.key(&key.into())?))
    }

    pub fn range<'b, S: IterableStorage>(
        &self,
        storage: &'b S,
        start: Bound<K>,
        end: Bound<K>,
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
        Ok(DsIter::new(self.prefix.to_vec(), iter))
    }
}

#[cfg(test)]
mod test {
    use std::collections::{BTreeMap, HashMap};

    use crate::{mock::DisplayEncoding, Item};

    use super::*;

    #[test]
    fn test_map() {
        type MapT = Map<'static, String, Item<'static, String, DisplayEncoding>>;
        const MAP: MapT = Map::new(b"foo");

        let mut storage: HashMap<Vec<u8>, Vec<u8>> = HashMap::new();
        let acc = MAP.at("bar").unwrap();
        assert_eq!(acc.may_load(&storage), Ok(None));
        acc.save(&mut storage, &"baz".to_string()).unwrap();
        assert_eq!(acc.may_load(&storage), Ok(Some("baz".to_string())));

        type NestedT = Map<'static, usize, MapT>;
        const NESTED: NestedT = Map::new(b"nested");
        let acc = NESTED.at(42usize).unwrap().at("qux").unwrap();
        assert_eq!(acc.may_load(&storage), Ok(None));
        acc.save(&mut storage, &"quux".to_string()).unwrap();
        assert_eq!(acc.may_load(&storage), Ok(Some("quux".to_string())));

        println!("{storage:?}");

        dbg!(std::any::type_name_of_val(&MAP));
        dbg!(std::any::type_name::<<MapT as DataStructure>::Key>());
        dbg!(std::any::type_name_of_val(&NESTED));
        dbg!(std::any::type_name::<<NestedT as DataStructure>::Key>());
    }

    #[test]
    fn test_map_iter() {
        let mut storage: BTreeMap<Vec<u8>, Vec<u8>> = BTreeMap::new();
        const MAP: Map<String, Item<String, DisplayEncoding>> = Map::new(b"foo");

        for i in 0..10 {
            let key = format!("k{}", i);
            let item = MAP.at(key.clone()).unwrap();
            item.save(&mut storage, &format!("value{}", i)).unwrap();
        }

        let iter = MAP
            .range(
                &storage,
                Bound::Unbounded,
                Bound::Unbounded,
                Order::Ascending,
            )
            .unwrap();
        for item in iter {
            if let Ok((key, value)) = item {
                println!("{:?}: {:?}", key, value);
            } else {
                panic!("Error: {:?}", item);
            }
        }
    }
}
