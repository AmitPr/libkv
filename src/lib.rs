#![allow(unused)]

mod map;
mod sealed;
mod trait_impls;
mod varint;

use std::marker::PhantomData;

pub use varint::to_varint_u128;

use sealed::*;

trait NodeType: Sealed {}

pub struct Leaf;
seal!(Leaf);
impl NodeType for Leaf {}

pub struct Branch;
seal!(Branch);
impl NodeType for Branch {}

trait Node {
    // TODO: Name this something better.
    type Category: NodeType;
    // TODO: Name this something better.
    type Key: Key;

    type Value;
}

trait Key {
    // TODO: Name this something better.
    type Size: SizeHint;

    fn encode(&self) -> Vec<u8>;

    /// Decode a key from a byte slice, consuming the bytes necessary to decode the key.
    fn decode(bytes: &mut &[u8]) -> Self;
}

pub trait SizeHint: Sealed {}

pub struct FixedSize<const SIZE: usize>;
impl<const SIZE: usize> Sealed for FixedSize<SIZE> {}
impl<const SIZE: usize> SizeHint for FixedSize<SIZE> {}

pub struct VariableSize;
seal!(VariableSize);
impl SizeHint for VariableSize {}

trait KeyEncoder {
    const RULE: EncodingRule;
}

enum EncodingRule {
    /// The remainder of the key is a single segment.
    Consume,
    /// The next segment of the key has a varint-prefixed length.
    Prefixed,
    /// The next segment of the key is a fixed number of bytes.
    Fixed(usize),
}

// A dynamic key only length-prefixes if it is used in a non-terminal context.
impl KeyEncoder for (VariableSize, Leaf) {
    const RULE: EncodingRule = EncodingRule::Consume;
}

// A dynamic key length-prefixes if it is used in a non-terminal context.
impl KeyEncoder for (VariableSize, Branch) {
    const RULE: EncodingRule = EncodingRule::Prefixed;
}

// A fixed-size key consumes if it is used in a terminal context.
impl<const SIZE: usize> KeyEncoder for (FixedSize<SIZE>, Leaf) {
    const RULE: EncodingRule = EncodingRule::Consume;
}

// A fixed-size key has a const-defined length if it is used in a non-terminal context.
impl<const SIZE: usize> KeyEncoder for (FixedSize<SIZE>, Branch) {
    const RULE: EncodingRule = EncodingRule::Fixed(SIZE);
}

// impl Key for String {
//     type Size = VariableSize;

//     fn encode(&self) -> Vec<u8> {
//         self.as_bytes().to_vec()
//     }
// }

struct CompoundKey<T: Key, U: Key>(T, U);
impl<K: Key> CompoundKey<K, ()> {
    pub fn new(key: K) -> Self {
        CompoundKey(key, ())
    }
}

impl<T: Key, U: Key> Key for CompoundKey<T, U> {
    type Size = VariableSize;

    fn encode(&self) -> Vec<u8> {
        let mut bytes = self.0.encode();
        bytes.extend_from_slice(&self.1.encode());
        bytes
    }

    fn decode(bytes: &mut &[u8]) -> Self {
        let key = T::decode(bytes);
        let unit = U::decode(bytes);
        CompoundKey(key, unit)
    }
}

impl<T: Key, U: Key> CompoundKey<T, U> {
    pub fn push<NewK: Key>(self, key: NewK) -> CompoundKey<T, CompoundKey<U, NewK>> {
        CompoundKey(self.0, CompoundKey(self.1, key))
    }
}

/// An Iterable Map built atop a KV store
pub struct Map<K: Key, V> {
    _marker: PhantomData<(K, V)>,
}
