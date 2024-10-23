use std::{
    convert::Infallible,
    marker::PhantomData,
    ops::{Bound, RangeBounds},
};

use disjoint_impls::disjoint_impls;

use crate::v2::{key::AsInner, storage::encode_bound};

use super::{
    key::{CompoundKey, DecodeResult, EncodeResult, Key, KeySegment, KeySerde},
    storage::{IterableStorage, RawKey},
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
}

pub struct Map<K, V>(PhantomData<(K, V)>);
pub struct Vector<T>(PhantomData<T>);
pub struct Item<T>(PhantomData<T>);

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

impl<K: KeySerde, V: Container> Container for Map<K, V> {
    type ContainerType = Branch<V>;
    type Key = KeySegment<K, Self>;
    type FullKey = CompoundKey<Self::Key, V::FullKey, Self>;
    type Value = V::Value;
}

impl<T: Container> Container for Vector<T> {
    type ContainerType = Branch<T>;
    type Key = KeySegment<usize, Self>;
    type FullKey = CompoundKey<Self::Key, T::FullKey, Self>;
    type Value = T::Value;
}

impl<T> Container for Item<T> {
    type ContainerType = Leaf<T>;
    type Key = KeySegment<(), Self>;
    type FullKey = Self::Key;
    type Value = T;
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn compiles() {
        type Map1 = Map<String, Item<()>>;
        type Map2 = Map<String, Map<String, Item<()>>>;
        type Vec1 = Vector<Item<()>>;
        type Vec2 = Vector<Vector<Item<()>>>;
        type VecMap = Vector<Map<String, Item<()>>>;
        type Item1 = Item<()>;

        println!("{}", std::any::type_name::<Map1>());
        println!("{}", std::any::type_name::<<Map2 as Container>::FullKey>());
    }
}
