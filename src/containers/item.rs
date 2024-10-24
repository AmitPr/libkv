use std::{borrow::Cow, marker::PhantomData};

use crate::{
    key::{KeySegment, KeySerde},
    serialization::{Decodable, Encodable, Encoding},
    storage::{Storage, StorageMut},
};

use super::traits::{Container, Leaf};

pub struct Item<'a, T, E: Encoding>(Cow<'a, [u8]>, pub PhantomData<(T, E)>);
impl<'a, T: Encodable<E> + Decodable<E>, E: Encoding> Container for Item<'a, T, E> {
    type ContainerType = Leaf<T, E>;
    type Key = KeySegment<Vec<u8>, Self>;
    type FullKey = Self::Key;
    type Value = T;
    type Encoding = E;
}
impl<'a, T: Encodable<E> + Decodable<E>, E: Encoding> Item<'a, T, E> {
    pub const fn from_bytes(bytes: &'a [u8]) -> Self {
        Self(Cow::Borrowed(bytes), PhantomData)
    }

    pub const fn from_vec(bytes: Vec<u8>) -> Self {
        Self(Cow::Owned(bytes), PhantomData)
    }
}

impl<'a, T: Encodable<E> + Decodable<E>, E: Encoding> Item<'a, T, E> {
    pub fn may_load<S: Storage>(&self, storage: &S) -> Result<Option<T>, E::DecodeError> {
        // TODO: Error propagation should be nice for Key / Value serialization errrors.
        let key = self
            .0
            .to_vec()
            .encode()
            .unwrap_or_else(|_| panic!("Failed to encode key"));
        let bytes = storage.get_raw(&key);
        bytes.map(|b| T::decode(b.as_slice())).transpose()
    }

    pub fn save<S: StorageMut>(&self, storage: &mut S, value: &T) -> Result<(), E::EncodeError> {
        let key = self
            .0
            .to_vec()
            .encode()
            .unwrap_or_else(|_| panic!("Failed to encode key"));
        let value = value.encode()?;
        storage.set_raw(key, value);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::mock::DisplayEncoding;

    use super::*;

    #[test]
    fn test_item() {
        const FOO: Item<String, DisplayEncoding> = Item::from_bytes(b"foo");
        const FOOBAR: Item<String, DisplayEncoding> = Item::from_bytes(b"foobar");

        let mut storage = std::collections::BTreeMap::new();

        assert_eq!(FOO.may_load(&storage).unwrap(), None);
        FOO.save(&mut storage, &"qux".to_string()).unwrap();
        assert_eq!(FOO.may_load(&storage).unwrap(), Some("qux".to_string()));

        assert_eq!(FOOBAR.may_load(&storage).unwrap(), None);
        FOOBAR.save(&mut storage, &"baz".to_string()).unwrap();
        assert_eq!(FOOBAR.may_load(&storage).unwrap(), Some("baz".to_string()));

        println!("{storage:?}");
    }
}
