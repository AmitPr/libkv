#![allow(unused)]

mod map;
mod sealed;
mod trait_impls;
mod varint;

use std::{fmt::Debug, marker::PhantomData};

use disjoint_impls::disjoint_impls;

use sealed::*;

pub trait Key: Debug {
    type Error;

    fn encode(&self) -> Vec<u8>;

    /// Decode a key from a byte slice, consuming the bytes necessary to decode the key.
    fn decode(bytes: &mut &[u8]) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

#[derive(Debug)]
pub struct CompoundKey<T: Key, U: Key>(T, U);
impl<K: Key> CompoundKey<K, ()> {
    pub fn new(key: K) -> Self {
        CompoundKey(key, ())
    }
}

impl<T: Key, U: Key> Key for CompoundKey<T, U> {
    type Error = (Option<T::Error>, Option<U::Error>);
    fn encode(&self) -> Vec<u8> {
        let mut bytes = self.0.encode();
        bytes.extend_from_slice(&self.1.encode());
        bytes
    }

    fn decode(bytes: &mut &[u8]) -> Result<Self, Self::Error> {
        let key = T::decode(bytes).map_err(|e| (Some(e), None))?;
        let unit = U::decode(bytes).map_err(|e| (None, Some(e)))?;
        Ok(CompoundKey(key, unit))
    }
}

impl<T: Key, U: Key> CompoundKey<T, U> {
    pub fn push<NewK: Key>(self, key: NewK) -> CompoundKey<T, CompoundKey<U, NewK>> {
        CompoundKey(self.0, CompoundKey(self.1, key))
    }
}

pub trait NodeType: Sealed {}

pub struct Leaf<V>(PhantomData<V>);
pub struct Branch<Inner: Node>(PhantomData<Inner>);
impl<V> Sealed for Leaf<V> {}
impl<V> NodeType for Leaf<V> {}
impl<Inner: Node> Sealed for Branch<Inner> {}
impl<Inner: Node> NodeType for Branch<Inner> {}

pub trait Node: NodeValue {
    type Category: NodeType;
    type Key: Key;
}

disjoint_impls! {
    pub trait NodeValue {
        type Value;
    }

    impl<N: Node<Category = Branch<M>>, M: Node + NodeValue> NodeValue for N {
        type Value = <M as NodeValue>::Value;
    }

    impl<N: Node<Category = Leaf<V>>, V> NodeValue for N {
        type Value = V;
    }
}

pub struct Item<V>(V);

impl<V> Node for Item<V> {
    type Category = Leaf<V>;
    type Key = ();
}

/// An Iterable Map built atop a KV store
pub struct Map<K: Key, V: Node> {
    _marker: PhantomData<(K, V)>,
}

impl<K: Key, V: Node<Category = Branch<M>>, M: Node> Node for Map<K, V> {
    type Category = Branch<V>;
    type Key = CompoundKey<K, V::Key>;
}

impl<K: Key, T> Node for Map<K, Item<T>> {
    type Category = Branch<Item<T>>;
    type Key = K;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_compiles() {
        type t = Map<String, Item<()>>;
        type t2 = Map<String, Map<String, Item<()>>>;

        println!("{}", std::any::type_name::<<t as Node>::Category>());
        println!("{}", std::any::type_name::<<t as Node>::Key>());
        println!("{}", std::any::type_name::<<t as NodeValue>::Value>());

        println!("{}", std::any::type_name::<<t2 as Node>::Category>());
        println!("{}", std::any::type_name::<<t2 as Node>::Key>());
        println!("{}", std::any::type_name::<<t2 as NodeValue>::Value>());

        let x: <t2 as Node>::Key = ("foo".to_string(), "bar".to_string()).into();
        println!("{:?}", x);
    }
}
