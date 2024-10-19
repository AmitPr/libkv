use std::marker::PhantomData;

use disjoint_impls::disjoint_impls;

use crate::key::{CompoundKey, Key};

use super::{Branch, Leaf, Node};

disjoint_impls! {
    pub trait Access: Node {
        type Inner;
        fn key(&self, key: impl Into<Self::KeySegment>) -> Accessor<Self::KeySegment, Self::Inner> {
            Accessor {
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
pub struct Accessor<K: Key, Inner> {
    pub partial: K,
    _marker: PhantomData<Inner>,
}

// Can only get and set if the value is a Leaf.
impl<K: Key, Inner: Node<Category = Leaf<V>>, V> Accessor<K, Inner> {
    fn get(&self) -> Option<V> {
        unimplemented!()
    }

    fn set(&self, value: V) {
        unimplemented!()
    }
}
// Otherwise, can keep chaining keys.
impl<K: Key, Inner: Node<Category = Branch<M>, KeySegment = K>, M: Node> Accessor<K, Inner> {
    pub fn key(
        self,
        key: impl Into<Inner::KeySegment>,
    ) -> Accessor<CompoundKey<K, Inner::KeySegment>, M> {
        Accessor {
            partial: CompoundKey(self.partial, key.into()),
            _marker: PhantomData,
        }
    }

    // TODO: Implement this for the node itself, and find a better name.
    pub fn full(
        self,
        key: impl Into<Inner::FullKey>,
    ) -> Accessor<CompoundKey<K, Inner::FullKey>, M::Leaf> {
        Accessor {
            partial: CompoundKey(self.partial, key.into()),
            _marker: PhantomData,
        }
    }
}

impl<K: Key, Inner> Key for Accessor<K, Inner> {
    type Error = K::Error;
    fn encode(&self) -> Vec<u8> {
        self.partial.encode()
    }

    fn decode(bytes: &mut &[u8]) -> Result<Self, Self::Error> {
        let partial = K::decode(bytes)?;
        Ok(Accessor {
            partial,
            _marker: PhantomData,
        })
    }
}
