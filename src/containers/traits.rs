use std::{
    convert::Infallible,
    marker::PhantomData,
    ops::{Bound, RangeBounds},
};

use disjoint_impls::disjoint_impls;

use crate::{key::AsInner, storage::encode_bound};

use crate::{
    key::{CompoundKey, DecodeResult, EncodeResult, Key, KeySegment, KeySerde},
    serialization::{Decodable, Encodable, Encoding},
    storage::{IterableStorage, RawKey, Storage, StorageMut},
};

mod sealed {
    pub trait ContainerTypeSeal {}
}
pub trait ContainerType: sealed::ContainerTypeSeal {}
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

pub struct Leaf<V, E>(PhantomData<(V, E)>);
impl<V, E> sealed::ContainerTypeSeal for Leaf<V, E> {}
impl<V, E> ContainerType for Leaf<V, E> {}

pub struct Branch<Inner>(PhantomData<Inner>);
impl<Inner: Container> sealed::ContainerTypeSeal for Branch<Inner> {}
impl<Inner: Container> ContainerType for Branch<Inner> {}

impl<T: Encodable<E> + Decodable<E>, E: Encoding> Container for Leaf<T, E> {
    type ContainerType = Leaf<T, E>;
    type Key = KeySegment<(), Self>;
    type FullKey = Self::Key;
    type Value = T;
    type Encoding = E;
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

    impl<C: Container<ContainerType = Leaf<V, E>, FullKey = K>, V,E: Encoding, K: KeySerde> PartialToInner for C {
        type ChildKey = Infallible;
        fn partial_to_inner(_key: &K::PartialKey) -> Option<&Self::ChildKey> {
            None
        }
    }
}
