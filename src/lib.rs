#![allow(unused)]

mod map;
mod sealed;
mod serialization;
mod trait_impls;

use std::{fmt::Debug, marker::PhantomData};

use disjoint_impls::disjoint_impls;

use sealed::*;

pub trait Key {
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

pub trait Node: NodeValue + Sized {
    type Category: NodeType;
    type KeySegment: Key;
    type FullKey: Key;

    // type Accessor: NodeAccessor<Self>;
}

trait NodeAccessor<N: Node> {
    fn get(&self, key: impl Into<N::KeySegment>) -> Option<N::Value>;
    fn set(&mut self, key: impl Into<N::KeySegment>, value: N::Value);
}

disjoint_impls! {
    pub trait NodeValue {
        type Leaf;
        type Value;
    }

    impl<N: Node<Category = Branch<M>>, M: Node + NodeValue> NodeValue for N {
        type Value = <M as NodeValue>::Value;
        type Leaf = <M as NodeValue>::Leaf;
    }

    impl<N: Node<Category = Leaf<V>>, V> NodeValue for N {
        type Value = V;
        type Leaf = N;
    }
}

#[derive(Debug)]
pub struct Item<V>(PhantomData<V>);

impl<V> Node for Item<V> {
    type Category = Leaf<V>;
    type KeySegment = ();
    type FullKey = ();
}

/// An Iterable Map built atop a KV store
pub struct Map<K: Key, V: Node> {
    _marker: PhantomData<(K, V)>,
}

impl<K: Key, V: Node<Category = Branch<M>>, M: Node> Node for Map<K, V> {
    type Category = Branch<V>;
    type KeySegment = K;
    type FullKey = CompoundKey<K, V::KeySegment>;
}

// Flatten Key if the Value is an Item.
impl<K: Key, T> Node for Map<K, Item<T>> {
    type Category = Branch<Item<T>>;
    type KeySegment = K;
    type FullKey = K;
}

/*
TODO: Working through Accessor semantics and DX.

How would I want to use an accessor?
let map: Map<String, Item<()>> = Map::new();
map.key("foo").get();
let map: Map<String, Map<String, Item<String>>> = Map::new();
map.key("foo").key("bar").get();
map.key(("foo", "bar")).get();
map.key("foo").key("bar").set("baz");

concretely:
Map<K,V>::key -> PartialKey<K, V>

*/

#[derive(Debug)]
struct PartialKey<K: Key, Inner> {
    partial: K,
    _marker: PhantomData<Inner>,
}

// Can only get and set if the value is a Leaf.
impl<K: Key, Inner: Node<Category = Leaf<V>>, V> PartialKey<K, Inner> {
    fn get(&self) -> Option<V> {
        unimplemented!()
    }

    fn set(&self, value: V) {
        unimplemented!()
    }
}
// Otherwise, can keep chaining keys.
impl<K: Key, Inner: Node<Category = Branch<M>, KeySegment = K>, M: Node> PartialKey<K, Inner> {
    fn key(
        self,
        key: impl Into<Inner::KeySegment>,
    ) -> PartialKey<CompoundKey<K, Inner::KeySegment>, M> {
        PartialKey {
            partial: CompoundKey(self.partial, key.into()),
            _marker: PhantomData,
        }
    }

    // TODO: Implement this for the node itself, and find a better name.
    fn full(
        self,
        key: impl Into<Inner::FullKey>,
    ) -> PartialKey<CompoundKey<K, Inner::FullKey>, M::Leaf> {
        PartialKey {
            partial: CompoundKey(self.partial, key.into()),
            _marker: PhantomData,
        }
    }
}

impl<K: Key, Inner> Key for PartialKey<K, Inner> {
    type Error = K::Error;
    fn encode(&self) -> Vec<u8> {
        self.partial.encode()
    }

    fn decode(bytes: &mut &[u8]) -> Result<Self, Self::Error> {
        let partial = K::decode(bytes)?;
        Ok(PartialKey {
            partial,
            _marker: PhantomData,
        })
    }
}

disjoint_impls! {
    trait Access: Node {
        type Inner;
        fn key(&self, key: impl Into<Self::KeySegment>) -> PartialKey<Self::KeySegment, Self::Inner> {
            PartialKey {
                partial: key.into(),
                _marker: PhantomData,
            }
        }
    }
    impl<N: Node<Category = Branch<M>>, M: Node> Access for N {
        type Inner = M;
    }
    impl<N: Node<Category = Leaf<V>>, V> Access for N {
        type Inner = N;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::type_name;

    #[test]
    fn test_map_compiles() {
        type Map1 = Map<String, Item<()>>;
        type Map2 = Map<String, Map<String, Item<()>>>;

        println!("{}", type_name::<<Map1 as Node>::Category>());
        println!("{}", type_name::<<Map1 as Node>::KeySegment>());
        println!("{}", type_name::<<Map1 as NodeValue>::Value>());

        println!("{}", type_name::<<Map2 as Node>::Category>());
        println!("{}", type_name::<<Map2 as Node>::KeySegment>());
        println!("{}", type_name::<<Map2 as NodeValue>::Value>());

        let x: <Map2 as Node>::FullKey = ("foo".to_string(), "bar".to_string()).into();
        println!("{:?}", x);

        let map: Map<String, Item<()>> = Map {
            _marker: PhantomData,
        };
        let acc = map.key("foo");
        println!("{:?} -> {:?}", acc, acc.encode());
        let map: Map<String, Map<String, Item<String>>> = Map {
            _marker: PhantomData,
        };
        let acc = map.key("foo");
        println!("{:?} -> {:?}", acc.partial, acc.encode());
        let acc = acc.key("bar");
        println!("{:?} -> {:?}", acc, acc.encode());
        let map: Map<String, Map<String, Map<String, Item<String>>>> = Map {
            _marker: PhantomData,
        };
        let acc = map.key("foo");
        println!("{:?} -> {:?}", acc.partial, acc.encode());
        let acc = acc.full(("bar".to_string(), "baz".to_string()));
        println!("{:?} -> {:?}", acc, acc.encode());
    }
}
