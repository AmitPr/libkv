use std::marker::PhantomData;

use crate::key::{CompoundKey, Key};

use super::{Branch, NoPre, Node};

/// An Iterable Map built atop a KV store
#[derive(Debug)]
pub struct Vector<T: Node, P: Key = ()> {
    prefix: P,
    _marker: PhantomData<T>,
}

// Only enable ::new() for unprefixed Maps (i.e. root nodes).
impl<T: Node> Vector<T> {
    pub const fn new() -> Self {
        Self {
            prefix: (),
            _marker: PhantomData,
        }
    }
}

impl<T: Node<Prefix: NoPre>, P: Key> Vector<T, P> {
    pub fn at(self, index: impl Into<usize>) -> <T as Node>::Prefixed<CompoundKey<P, usize>> {
        <T as Node>::with_prefix(CompoundKey::new(self.prefix, index.into()))
    }
}

impl<T: Node, P: Key> Node for Vector<T, P> {
    type Category = Branch<T>;
    type KeySegment = usize;
    type FullKey = CompoundKey<usize, T::KeySegment>;
    type Prefixed<Pre: Key> = Vector<T, Pre>;
    type Prefix = P;

    fn with_prefix<Pre: Key>(prefix: Pre) -> Self::Prefixed<Pre> {
        Vector {
            prefix,
            _marker: PhantomData,
        }
    }
}
