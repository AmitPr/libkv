use std::marker::PhantomData;

use super::{Leaf, Node};

#[derive(Debug)]
pub struct Item<V>(PhantomData<V>);

impl<V> Node for Item<V> {
    type Category = Leaf<V>;
    type KeySegment = ();
    type FullKey = ();
}
