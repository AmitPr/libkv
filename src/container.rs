use std::{borrow::Cow, marker::PhantomData, ops::Bound};

use crate::serialization::{Codec, Decodable, Encoding};

use super::{
    storage::{Iter, IterableStorage, Storage, StorageMut},
    KeyDeserializeError, KeySerde, KeySerializeError, StorageError,
};

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

enum KeyType<K: KeySerde> {
    Raw(Vec<u8>),
    Key(K),
}
impl<K: KeySerde> KeySerde for KeyType<K> {
    fn encode(&self) -> Result<Vec<u8>, KeySerializeError> {
        match self {
            Self::Raw(key) => Ok(key.clone()),
            Self::Key(key) => key.encode(),
        }
    }

    fn decode(bytes: &mut &[u8]) -> Result<Self, KeyDeserializeError> {
        K::decode(bytes).map(Self::Key)
    }
}

pub struct Item<'a, V: Codec<Enc>, Enc: Encoding, K: KeySerde = Cow<'a, [u8]>>(
    KeyType<K>,
    PhantomData<(&'a K, V, Enc)>,
);
impl<'a, V: Codec<Enc>, Enc: Encoding, K: KeySerde> DataStructure for Item<'a, V, Enc, K> {
    type Key = K;
    type Enc = Enc;
    type Value = V;
    type DsType = Terminal;

    fn with_prefix(prefix: Vec<u8>) -> Self {
        Self(KeyType::Raw(prefix), PhantomData)
    }

    fn should_skip_key(_: &Self::Key) -> bool {
        false
    }
}

impl<V: Codec<Enc>, Enc: Encoding> Item<'static, V, Enc> {
    pub const fn new(key: &'static [u8]) -> Self {
        Self(KeyType::Key(Cow::Borrowed(key)), PhantomData)
    }
}

impl<'a, V: Codec<Enc>, Enc: Encoding, K: KeySerde> Item<'a, V, Enc, K> {
    pub fn with_key(key: K) -> Self {
        Self(KeyType::Key(key), PhantomData)
    }

    pub fn may_load<S: Storage>(&self, storage: &S) -> Result<Option<V>, StorageError<Enc>> {
        let bytes = storage.get(&self.0)?;
        let value = bytes.map(|b| V::decode(b.as_slice())).transpose();
        value.map_err(StorageError::ValueDeserialize)
    }

    pub fn save<S: StorageMut>(&self, storage: &mut S, value: &V) -> Result<(), StorageError<Enc>> {
        let key = self.0.encode()?;
        let value = value.encode().map_err(StorageError::ValueSerialize)?;
        storage.set_raw(key, value);
        Ok(())
    }
}

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
    fn key(&self, key: K) -> Result<Vec<u8>, KeySerializeError> {
        let encoded = key.encode()?;
        let full = [self.prefix.as_ref(), &encoded].concat();

        Ok(full)
    }
    pub fn at(&self, key: impl Into<K>) -> Result<V, KeySerializeError> {
        Ok(V::with_prefix(self.key(key.into())?))
    }

    pub fn range<'b, S: IterableStorage>(
        &self,
        storage: &'b S,
        start: Bound<K>,
        end: Bound<K>,
    ) -> Result<DsIter<'b, Self>, KeySerializeError> {
        let start = match start {
            Bound::Included(k) => Bound::Included(self.key(k)?),
            Bound::Excluded(k) => Bound::Excluded(self.key(k)?),
            Bound::Unbounded => Bound::Unbounded,
        };
        let end = match end {
            Bound::Included(k) => Bound::Included(self.key(k)?),
            Bound::Excluded(k) => Bound::Excluded(self.key(k)?),
            Bound::Unbounded => Bound::Unbounded,
        };
        let iter = storage.iter(start, end)?;
        Ok(DsIter::new(self.prefix.to_vec(), iter))
    }
}

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

#[cfg(test)]
mod test {
    use std::collections::{BTreeMap, HashMap};

    use crate::mock::DisplayEncoding;

    use super::*;

    #[test]
    fn test_item() {
        const ITEM: Item<String, DisplayEncoding> = Item::new(b"foo");
        let item: Item<String, DisplayEncoding> = Item::with_key(Cow::Owned(b"bar".to_vec()));

        let mut storage: HashMap<Vec<u8>, Vec<u8>> = HashMap::new();
        assert_eq!(ITEM.may_load(&storage), Ok(None));
        assert_eq!(item.may_load(&storage), Ok(None));

        ITEM.save(&mut storage, &"baz".to_string()).unwrap();
        assert_eq!(ITEM.may_load(&storage), Ok(Some("baz".to_string())));
        assert_eq!(item.may_load(&storage), Ok(None));

        item.save(&mut storage, &"qux".to_string()).unwrap();
        assert_eq!(ITEM.may_load(&storage), Ok(Some("baz".to_string())));
        assert_eq!(item.may_load(&storage), Ok(Some("qux".to_string())));
    }

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
            .range(&storage, Bound::Unbounded, Bound::Unbounded)
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
