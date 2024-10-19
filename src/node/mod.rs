mod access;
mod item;
mod map;
mod nodetype;
pub use {
    access::{Access, Accessor, AccessorT},
    item::Item,
    map::Map,
    nodetype::*,
};

use crate::key::Key;
use disjoint_impls::disjoint_impls;
use std::marker::PhantomData;

pub trait Node: NodeValue + Sized {
    type Category: NodeType;
    /// The key type that this node is responsible for.
    type KeySegment: Key;
    /// The full key, composing of all the keys from this node to the leaf.
    type FullKey: Key;

    type Prefixed<P: Key>: Node;

    // TODO: Restrict with_prefix somehow.
    // Effectively, this would prevent the user from accidentally
    // discovering / overwriting via with_prefix during access.
    // i.e., with_prefix shouldn't show in intellisense.
    fn with_prefix<Pre: Key>(prefix: Pre) -> Self::Prefixed<Pre>;
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
