use std::marker::PhantomData;

use crate::v2::{
    key::{CompoundKey, KeySegment, KeySerde},
    serialization::{Decodable, Encodable, Encoding},
    storage::{Storage, StorageMut},
};

use super::traits::{Branch, Container, Leaf};

pub struct Map<K, V, E: Encoding>(pub PhantomData<(K, V, E)>);
impl<K: KeySerde, V: Container, E: Encoding> Container for Map<K, V, E> {
    type ContainerType = Branch<V>;
    type Key = KeySegment<K, Self>;
    type FullKey = CompoundKey<Self::Key, V::FullKey, Self>;
    type Value = V::Value;
    type Encoding = E;
}

impl<K: KeySerde, V: Container, E: Encoding> Map<K, V, E>
where
    Self: Container,
    <Self as Container>::Value: Encodable<E> + Decodable<E>,
{
    pub fn key(&self, key: impl Into<<Self as Container>::Key>) -> <Self as Container>::Key {
        key.into()
    }

    pub fn may_load<S: Storage>(
        &self,
        storage: &S,
        key: &<Self as Container>::FullKey,
    ) -> Result<Option<<Self as Container>::Value>, E::DecodeError> {
        let key_bytes = key
            .encode()
            .unwrap_or_else(|_| panic!("Failed to encode key"));
        let value_bytes = storage.get_raw(&key_bytes);
        value_bytes
            .map(|b| <Self as Container>::Value::decode(b.as_slice()))
            .transpose()
    }

    pub fn save<S: StorageMut>(
        &self,
        storage: &mut S,
        key: &<Self as Container>::FullKey,
        value: &<Self as Container>::Value,
    ) -> Result<(), E::EncodeError> {
        let key_bytes = key
            .encode()
            .unwrap_or_else(|_| panic!("Failed to encode key"));
        let value_bytes = value.encode()?;
        storage.set_raw(key_bytes, value_bytes);
        Ok(())
    }
}
