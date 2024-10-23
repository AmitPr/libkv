use std::marker::PhantomData;

use crate::{
    key::{CompoundKey, KeySegment, KeySerde},
    serialization::{Decodable, Encodable, Encoding},
    storage::{Storage, StorageMut},
};

use super::traits::{Branch, Container, Leaf};

pub struct Map<K, V>(pub PhantomData<(K, V)>);
impl<K: KeySerde, V: Container> Container for Map<K, V> {
    type ContainerType = Branch<V>;
    type Key = KeySegment<K, Self>;
    type FullKey = CompoundKey<Self::Key, V::FullKey, Self>;
    type Value = V::Value;
    type Encoding = V::Encoding;
}

impl<K: KeySerde, V: Container> Map<K, V>
where
    Self: Container,
    <Self as Container>::Value:
        Encodable<<Self as Container>::Encoding> + Decodable<<Self as Container>::Encoding>,
{
    pub fn key(&self, key: impl Into<<Self as Container>::Key>) -> <Self as Container>::Key {
        key.into()
    }

    pub fn may_load<S: Storage>(
        &self,
        storage: &S,
        key: &<Self as Container>::FullKey,
    ) -> Result<
        Option<<Self as Container>::Value>,
        <<Self as Container>::Encoding as Encoding>::DecodeError,
    > {
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
    ) -> Result<(), <<Self as Container>::Encoding as Encoding>::EncodeError> {
        let key_bytes = key
            .encode()
            .unwrap_or_else(|_| panic!("Failed to encode key"));
        let value_bytes = value.encode()?;
        storage.set_raw(key_bytes, value_bytes);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::mock::DisplayEncoding;

    use super::*;

    #[test]
    fn test_map() {
        const MAP: Map<String, Leaf<String, DisplayEncoding>> = Map(PhantomData);

        let mut storage = std::collections::BTreeMap::new();

        let key = MAP.key("foo".to_string());
        assert_eq!(MAP.may_load(&storage, &key).unwrap(), None);
        MAP.save(&mut storage, &key, &"qux".to_string()).unwrap();

        let key = MAP.key(MAP.key(b"foo"), "bar");
        assert_eq!(MAP.may_load(&storage, &key).unwrap(), None);
        MAP.save(&mut storage, &key, &"qux".to_string()).unwrap();
        assert_eq!(
            MAP.may_load(&storage, &key).unwrap(),
            Some("qux".to_string())
        );
    }
}
