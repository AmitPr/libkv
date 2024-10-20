use std::marker::PhantomData;

use super::Node;
use crate::NodeTypeSeal;

pub trait NodeType: NodeTypeSeal {}

pub struct Leaf<V>(PhantomData<V>);
pub struct Branch<Inner: Node>(PhantomData<Inner>);
impl<V> NodeTypeSeal for Leaf<V> {}
impl<V> NodeType for Leaf<V> {}
impl<Inner: Node> NodeTypeSeal for Branch<Inner> {}
impl<Inner: Node> NodeType for Branch<Inner> {}
