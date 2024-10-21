use std::marker::PhantomData;

use crate::key::{CompoundKey, Key};

use super::{item::Item, Branch, NoPre, Node};

/// An Iterable Map built atop a KV store
#[derive(Debug)]
pub struct Map<K: Key, V: Node, P: Key = ()> {
    prefix: P,
    _marker: PhantomData<(K, V)>,
}

impl<K: Key, V: Node> Default for Map<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

// Only enable ::new() for unprefixed Maps (i.e. root nodes).
impl<K: Key, V: Node> Map<K, V> {
    pub const fn new() -> Self {
        Self {
            prefix: (),
            _marker: PhantomData,
        }
    }
}

impl<K: Key, V: Node<Prefix: NoPre>, P: Key> Map<K, V, P> {
    pub fn key(self, key: impl Into<K>) -> <V as Node>::Prefixed<CompoundKey<P, K>> {
        <V as Node>::with_prefix(CompoundKey::new(self.prefix, key.into()))
    }
}

impl<K: Key, V: Node, P: Key> Node for Map<K, V, P> {
    type Category = Branch<V>;
    type KeySegment = K;
    type FullKey = CompoundKey<K, V::KeySegment>;
    type Prefixed<Pre: Key> = Map<K, V, Pre>;
    type Prefix = P;

    fn with_prefix<Pre: Key>(prefix: Pre) -> Self::Prefixed<Pre> {
        Map {
            prefix,
            _marker: PhantomData,
        }
    }
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
