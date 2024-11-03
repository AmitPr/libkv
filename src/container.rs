use std::marker::PhantomData;

use crate::{Codec, Decodable, Encoding, Iter, KeyEncoding, StorageError};

/// Trait representing an arbitrary data structure that can be used in a
/// Key-Value store.
///
/// The trait requires the following associated types:
/// - `Key`: The type of the key used to index the data structure.
/// - `Enc`: The encoding used to serialize and deserialize the data structure.
/// - `Value`: The value type, that can be (de)serialized using the `Enc` encoding.
/// - `DsType`: A type that indicates whether the data structure is terminal or
///     non-terminal.
///
/// The trait also requires the following methods:
/// - `with_prefix`: A constructor that takes a byte-prefix, and returns a handle
///    to the data structure with that prefix.
/// - `should_skip_key`: A method that takes a key and returns whether the key
///   should be skipped when iterating over the data structure.
pub trait DataStructure {
    /// The type of the key used to index the data structure.
    type Key: Codec<KeyEncoding>;
    /// The encoding used to serialize and deserialize the data structure.
    type Enc: Encoding;
    /// The value type, that can be (de)serialized using the `Enc` encoding.
    type Value: Codec<Self::Enc>;
    /// A type that indicates whether the data structure is terminal or non-terminal.
    type DsType: sealed::ContainerType;

    /// Constructor that takes a byte-prefix, and returns a handle to the data structure
    fn with_prefix(prefix: Vec<u8>) -> Self
    where
        Self: Sized;

    /// Returns whether the given key should be skipped when iterating over the data structure.
    fn should_skip_key(key: &Self::Key) -> bool;
}

/// A trait representing a data structure that is a container of other data structures.
///
/// Most data structures are non-terminal, meaning they contain other data structures.
/// Since we represent actual values as the `Item` structure, even "simple" data structures
/// like a Key-Value Map are considered non-terminal, and thus are marked as `Container`.
pub trait Container: DataStructure<DsType = NonTerminal> {
    type Inner: DataStructure;
}

mod sealed {
    pub trait ContainerType: Sized {}
}

pub struct Terminal {}
impl sealed::ContainerType for Terminal {}
pub struct NonTerminal {}
impl sealed::ContainerType for NonTerminal {}

pub struct DsIter<'a, D: DataStructure> {
    _marker: PhantomData<D>,
    prefix: Vec<u8>,
    iter: Iter<'a, (Vec<u8>, Vec<u8>)>,
}

impl<'a, D: DataStructure> DsIter<'a, D> {
    pub const fn new(prefix: Vec<u8>, iter: Iter<'a, (Vec<u8>, Vec<u8>)>) -> Self {
        Self {
            _marker: PhantomData,
            prefix,
            iter,
        }
    }
}

impl<'a, D: DataStructure> Iterator for DsIter<'a, D> {
    type Item = Result<(D::Key, D::Value), StorageError<D::Enc>>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (mut key_bytes, val_bytes) = self.iter.next()?;
            if !key_bytes.starts_with(&self.prefix) {
                return None;
            }
            let key_bytes = key_bytes.split_off(self.prefix.len());
            let key = <<D as DataStructure>::Key as Decodable<KeyEncoding>>::decode(
                &mut key_bytes.as_slice(),
            )
            .map(|k| (D::should_skip_key(&k), k));

            match key {
                Ok((true, _)) => continue,
                Ok((false, key)) => {
                    let val = <D::Value as Decodable<D::Enc>>::decode(&mut val_bytes.as_slice())
                        .map_err(StorageError::ValueDeserialize);
                    return Some(val.map(|val| (key, val)));
                }
                Err(e) => return Some(Err(StorageError::KeyDeserialize(e))),
            }
        }
    }
}
