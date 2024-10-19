use std::marker::PhantomData;

use crate::key::Key;

use super::{Leaf, Node};

#[derive(Debug)]
pub struct Item<V, P: Key = ()> {
    prefix: P,
    _marker: PhantomData<V>,
}

// Only manually construct items if no prefix (root).
impl<V> Item<V> {
    pub const fn new<K: Key>(key: K) -> <Self as Node>::Prefixed<K> {
        Item::<V, K> {
            prefix: key,
            _marker: PhantomData,
        }
    }
}

impl<V, P: Key> Item<V, P> {
    // TODO, should actually access a storage layer.
    pub fn get(&self) -> Option<V> {
        let key = self.prefix.encode();
        println!("Getting key: {:?}", key);
        todo!()
    }
}

impl<V, P: Key> Node for Item<V, P> {
    type Category = Leaf<V>;
    type KeySegment = ();
    type FullKey = ();

    type Prefixed<Pre: Key> = Item<V, Pre>;
    fn with_prefix<Pre: Key>(prefix: Pre) -> Self::Prefixed<Pre> {
        Item {
            prefix,
            _marker: PhantomData,
        }
    }
}
