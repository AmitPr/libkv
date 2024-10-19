mod access;
mod item;
mod map;
mod nodetype;
pub use {
    access::{Access, PartialKey},
    item::Item,
    map::Map,
    nodetype::*,
};

use crate::{key::Key};
use disjoint_impls::disjoint_impls;
use std::marker::PhantomData;

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
