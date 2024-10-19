use std::marker::PhantomData;

use crate::key::{CompoundKey, Key};

use super::{item::Item, Branch, Node};

/// An Iterable Map built atop a KV store
#[derive(Debug, Default)]
pub struct Map<K: Key, V: Node>(PhantomData<(K, V)>);

impl<K: Key, V: Node> Map<K, V> {
    pub const fn new() -> Self {
        Self(PhantomData)
    }
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
