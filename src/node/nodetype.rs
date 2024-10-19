use std::marker::PhantomData;

use super::Node;
use crate::Sealed;

pub trait NodeType: Sealed {}

pub struct Leaf<V>(PhantomData<V>);
pub struct Branch<Inner: Node>(PhantomData<Inner>);
impl<V> Sealed for Leaf<V> {}
impl<V> NodeType for Leaf<V> {}
impl<Inner: Node> Sealed for Branch<Inner> {}
impl<Inner: Node> NodeType for Branch<Inner> {}
