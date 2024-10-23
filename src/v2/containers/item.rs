use std::marker::PhantomData;

use crate::v2::{
    key::{KeySegment, KeySerde},
    serialization::{Decodable, Encodable, Encoding},
    storage::{Storage, StorageMut},
};

use super::traits::{Container, Leaf};

pub struct Item<K: KeySerde, T, E: Encoding>(pub K, pub PhantomData<(T, E)>);
impl<T: Encodable<E> + Decodable<E>, E: Encoding, K: KeySerde> Container for Item<K, T, E> {
    type ContainerType = Leaf<T>;
    type Key = KeySegment<K, Self>;
    type FullKey = Self::Key;
    type Value = T;
    type Encoding = E;
}

impl<T: Encodable<E> + Decodable<E>, E: Encoding, K: KeySerde> Item<K, T, E> {
    pub fn may_load<S: Storage>(&self, storage: &S) -> Result<Option<T>, E::DecodeError> {
        // TODO: Error propagation should be nice for Key / Value serialization errrors.
        let key = self
            .0
            .encode()
            .unwrap_or_else(|_| panic!("Failed to encode key"));
        let bytes = storage.get_raw(&key);
        bytes.map(|b| T::decode(b.as_slice())).transpose()
    }

    pub fn save<S: StorageMut>(&self, storage: &mut S, value: &T) -> Result<(), E::EncodeError> {
        let key = self
            .0
            .encode()
            .unwrap_or_else(|_| panic!("Failed to encode key"));
        let value = value.encode()?;
        storage.set_raw(key, value);
        Ok(())
    }
}
