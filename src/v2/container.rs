use std::{
    convert::Infallible,
    marker::PhantomData,
    ops::{Bound, RangeBounds},
};

use disjoint_impls::disjoint_impls;

use crate::v2::{key::AsInner, storage::encode_bound};

use super::{
    key::{CompoundKey, DecodeResult, EncodeResult, Key, KeySegment, KeySerde},
    serialization::{Decodable, Encodable, Encoding},
    storage::{IterableStorage, RawKey, Storage, StorageMut},
};

mod sealed {
    pub trait ContainerTypeSeal {}
}
pub trait ContainerType: sealed::ContainerTypeSeal {}

pub struct Leaf<V>(PhantomData<V>);
pub struct Branch<Inner>(PhantomData<Inner>);
impl<V> sealed::ContainerTypeSeal for Leaf<V> {}
impl<V> ContainerType for Leaf<V> {}
impl<Inner: Container> sealed::ContainerTypeSeal for Branch<Inner> {}
impl<Inner: Container> ContainerType for Branch<Inner> {}

/// Containers are compile-time types that lay out a specification for how to
/// store and access data. They contain no data themselves, but simply define
/// access (lookup, iteration, etc.) and mutation (insertion, deletion,
/// modification, etc.) operations.
pub trait Container {
    type ContainerType: ContainerType;
    /// The key that this container is responsible for.
    type Key: Key<Container = Self>;
    // /// The full key, composing of all the keys from this container to the leaf.
    type FullKey: Key<Container = Self>;
    type Value;
    type Encoding: Encoding;
}

pub struct Vector<T, E: Encoding>(PhantomData<(T, E)>);

disjoint_impls! {
    /// A private trait to convert a container's partial key to the partial key
    /// of its inner container.
    pub(crate) trait PartialToInner: Container {
        type ChildKey;
        fn partial_to_inner(key: &<Self::FullKey as KeySerde>::PartialKey) -> Option<&Self::ChildKey>;
    }

    impl<
            C: Container<ContainerType = Branch<Inner>, FullKey = FK>, /* Container is a branch */
            FK: KeySerde<PartialKey = PK>, /* Container's FullKey has PartialKey = PK */
            Inner: Container<FullKey = CFK>, /* Branch's FullKey = CFK */
            PK: KeySerde + AsInner<CPK>, /* PK can be converted to CPK */
            CFK: KeySerde<PartialKey = CPK>, /* Child's FullKey has PartialKey = CPK */
            CPK: KeySerde, /* Child's PartialKey */
        > PartialToInner for C
    {
        type ChildKey = CPK;
        fn partial_to_inner(key: &PK) -> Option<&CPK> {
            key.as_inner_key()
        }
    }

    impl<C: Container<ContainerType = Leaf<V>, FullKey = K>, V, K: KeySerde> PartialToInner for C {
        type ChildKey = Infallible;
        fn partial_to_inner(_key: &K::PartialKey) -> Option<&Self::ChildKey> {
            None
        }
    }
}

pub struct Item<K: KeySerde, T, E: Encoding>(K, PhantomData<(T, E)>);
impl<T: Encodable<E> + Decodable<E>, E: Encoding, K: KeySerde> Container for Item<K, T, E> {
    type ContainerType = Leaf<T>;
    type Key = KeySegment<K, Self>;
    type FullKey = Self::Key;
    type Value = T;
    type Encoding = E;
}

impl<T: Encodable<E> + Decodable<E>, E: Encoding, K: KeySerde> Item<K, T, E> {
    pub fn may_load<S: Storage>(&self, storage: &S) -> Result<Option<T>, E::DecodeError> {
        // TODO: Error propagation should be nice for Key / Value serialization errrors.
        let key = self
            .0
            .encode()
            .unwrap_or_else(|_| panic!("Failed to encode key"));
        let bytes = storage.get_raw(&key);
        bytes.map(|b| T::decode(b.as_slice())).transpose()
    }

    pub fn save<S: StorageMut>(&self, storage: &mut S, value: &T) -> Result<(), E::EncodeError> {
        let key = self
            .0
            .encode()
            .unwrap_or_else(|_| panic!("Failed to encode key"));
        let value = value.encode()?;
        storage.set_raw(key, value);
        Ok(())
    }
}

pub struct Map<K, V, E: Encoding>(PhantomData<(K, V, E)>);
impl<K: KeySerde, V: Container, E: Encoding> Container for Map<K, V, E> {
    type ContainerType = Branch<V>;
    type Key = KeySegment<K, Self>;
    type FullKey = CompoundKey<Self::Key, V::FullKey, Self>;
    type Value = V::Value;
    type Encoding = E;
}

impl<K: KeySerde, V: Container, E: Encoding> Map<K, V, E>
where
    Self: Container,
    <Self as Container>::Value: Encodable<E> + Decodable<E>,
{
    pub fn key(&self, key: impl Into<<Self as Container>::Key>) -> <Self as Container>::Key {
        key.into()
    }

    pub fn may_load<S: Storage>(
        &self,
        storage: &S,
        key: &<Self as Container>::FullKey,
    ) -> Result<Option<<Self as Container>::Value>, E::DecodeError> {
        let key_bytes = key
            .encode()
            .unwrap_or_else(|_| panic!("Failed to encode key"));
        let value_bytes = storage.get_raw(&key_bytes);
        value_bytes
            .map(|b| <Self as Container>::Value::decode(b.as_slice()))
            .transpose()
    }

    pub fn save<S: StorageMut>(
        &self,
        storage: &mut S,
        key: &<Self as Container>::FullKey,
        value: &<Self as Container>::Value,
    ) -> Result<(), E::EncodeError> {
        let key_bytes = key
            .encode()
            .unwrap_or_else(|_| panic!("Failed to encode key"));
        let value_bytes = value.encode()?;
        storage.set_raw(key_bytes, value_bytes);
        Ok(())
    }
}

impl<T: Container, E: Encoding> Container for Vector<T, E> {
    type ContainerType = Branch<T>;
    type Key = KeySegment<usize, Self>;
    type FullKey = CompoundKey<Self::Key, T::FullKey, Self>;
    type Value = T::Value;
    type Encoding = E;
}

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;

    use crate::v2::mock::DisplayEncoding;

    use super::*;
    #[test]
    fn compiles() {
        // type Map1 = Map<String, Item<String, DisplayEncoding>, DisplayEncoding>;
        // type Map2 = Map<
        //     String,
        //     Map<String, Item<String, DisplayEncoding>, DisplayEncoding>,
        //     DisplayEncoding,
        // >;
        // type Vec1 = Vector<Item<String, DisplayEncoding>, DisplayEncoding>;
        // type Vec2 = Vector<Vector<Item<String, DisplayEncoding>, DisplayEncoding>, DisplayEncoding>;
        // type VecMap =
        //     Vector<Map<String, Item<String, DisplayEncoding>, DisplayEncoding>, DisplayEncoding>;
        // type Item1 = Item<String, DisplayEncoding>;

        // println!("{}", std::any::type_name::<Map1>());
        // println!("{}", std::any::type_name::<<Map2 as Container>::FullKey>());
    }

    #[test]
    fn test_item() {
        let mut storage: BTreeMap<Vec<u8>, Vec<u8>> = BTreeMap::new();
        let item: Item<String, String, DisplayEncoding> = Item("foo".to_string(), PhantomData);

        assert_eq!(item.may_load(&storage).unwrap(), None);
        item.save(&mut storage, &"bar".to_string()).unwrap();
        assert_eq!(item.may_load(&storage).unwrap(), Some("bar".to_string()));

        let item2 = Item("foobar".to_string(), PhantomData);
        assert_eq!(item2.may_load(&storage).unwrap(), None);
        item2.save(&mut storage, &"baz".to_string()).unwrap();
        assert_eq!(item2.may_load(&storage).unwrap(), Some("baz".to_string()));

        println!("{:?}", storage);
    }

    #[test]
    fn test_map() {
        let mut storage: BTreeMap<Vec<u8>, Vec<u8>> = BTreeMap::new();
        let map: Map<String, Item<(), String, DisplayEncoding>, DisplayEncoding> = Map(PhantomData);

        let key = map.key("foo".to_string());

        // let key = CompoundKey::new("foo".to_string(), "bar".to_string());
        // assert_eq!(map.may_load(&storage, &key).unwrap(), None);
        // map.save(&mut storage, &key, &"baz".to_string()).unwrap();
        // assert_eq!(
        //     map.may_load(&storage, &key).unwrap(),
        //     Some("baz".to_string())
        // );

        // let key2 = CompoundKey::new("foo".to_string(), "baz".to_string());
        // assert_eq!(map.may_load(&storage, &key2).unwrap(), None);
        // map.save(&mut storage, &key2, &"qux".to_string()).unwrap();
        // assert_eq!(
        //     map.may_load(&storage, &key2).unwrap(),
        //     Some("qux".to_string())
        // );

        // println!("{:?}", storage);
    }
}
