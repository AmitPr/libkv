#![allow(unused)]

mod key;
mod map;
mod node;
mod sealed;
mod serialization;
mod trait_impls;

pub use node::{Access, Branch, Item, Leaf, Map, Node, NodeValue, PartialKey};

use std::{fmt::Debug, marker::PhantomData};

use disjoint_impls::disjoint_impls;

use key::{CompoundKey, Key};
use sealed::*;
#[cfg(test)]
mod tests {
    use super::*;
    use std::any::type_name;

    #[test]
    fn test_map_compiles() {
        type Map1 = Map<String, Item<()>>;
        type Map2 = Map<String, Map<String, Item<()>>>;

        println!("{}", type_name::<<Map1 as Node>::Category>());
        println!("{}", type_name::<<Map1 as Node>::KeySegment>());
        println!("{}", type_name::<<Map1 as NodeValue>::Value>());

        println!("{}", type_name::<<Map2 as Node>::Category>());
        println!("{}", type_name::<<Map2 as Node>::KeySegment>());
        println!("{}", type_name::<<Map2 as NodeValue>::Value>());

        let x: <Map2 as Node>::FullKey = ("foo".to_string(), "bar".to_string()).into();
        println!("{:?}", x);

        let map: Map<String, Item<()>> = Map::new();
        let acc = map.key("foo");
        println!("{:?} -> {:?}", acc, acc.encode());
        let map: Map<String, Map<String, Item<String>>> = Map::new();
        let acc = map.key("foo");
        println!("{:?} -> {:?}", acc.partial, acc.encode());
        let acc = acc.key("bar");
        println!("{:?} -> {:?}", acc, acc.encode());
        let map: Map<String, Map<String, Map<String, Item<String>>>> = Map::new();
        let acc = map.key("foo");
        println!("{:?} -> {:?}", acc.partial, acc.encode());
        let acc = acc.full(("bar".to_string(), "baz".to_string()));
        println!("{:?} -> {:?}", acc, acc.encode());
    }
}
