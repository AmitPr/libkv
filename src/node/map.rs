use std::marker::PhantomData;

use crate::key::{CompoundKey, Key};

use super::{item::Item, Branch, Node};

/// An Iterable Map built atop a KV store
#[derive(Debug)]
pub struct Map<K: Key, V: Node, P: Key = ()> {
    prefix: P,
    _marker: PhantomData<(K, V)>,
}

impl<K: Key, V: Node> Default for Map<K, V> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

// Only enable ::new() for unprefixed Maps (i.e. root nodes).
impl<K: Key, V: Node> Map<K, V> {
    #[inline]
    pub const fn new() -> Self {
        Self {
            prefix: (),
            _marker: PhantomData,
        }
    }
}

impl<K: Key, V: Node, P: Key> Map<K, V, P> {
    pub fn key(self, key: impl Into<K>) -> <V as Node>::Prefixed<CompoundKey<P, K>> {
        <V as Node>::with_prefix(CompoundKey(self.prefix, key.into()))
    }
}

impl<K: Key, V: Node<Category = Branch<M>>, M: Node, Prefix: Key> Node for Map<K, V, Prefix> {
    type Category = Branch<V>;
    type KeySegment = K;
    type FullKey = CompoundKey<K, V::KeySegment>;
    type Prefixed<P: Key> = Map<K, V, P>;

    fn with_prefix<Pre: Key>(prefix: Pre) -> Self::Prefixed<Pre> {
        Map {
            prefix,
            _marker: PhantomData,
        }
    }
}

// Flatten Key if the Value is an Item.
impl<K: Key, T, P: Key> Node for Map<K, Item<T>, P> {
    type Category = Branch<Item<T>>;
    type KeySegment = K;
    type FullKey = K;
    type Prefixed<Pre: Key> = Map<K, Item<T>, Pre>;

    fn with_prefix<Pre: Key>(prefix: Pre) -> Self::Prefixed<Pre> {
        Map {
            prefix,
            _marker: PhantomData,
        }
    }
}

pub struct MapAccessor<N: Node, K: Key> {
    pub partial: K,
    _marker: PhantomData<N>,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_map() {
        const MAP: Map<String, Item<()>> = Map::new();
        let access = MAP.key("foo");
        println!("{:?}", access);
    }
}
