use std::{borrow::Cow, marker::PhantomData};

use crate::{
    Codec, DataStructure, Encoding, KeySerde, KeyType, Storage, StorageError, StorageMut, Terminal,
};

pub struct Item<'a, V: Codec<Enc>, Enc: Encoding, K: KeySerde = Cow<'a, [u8]>>(
    KeyType<K>,
    PhantomData<(&'a K, V, Enc)>,
);
impl<'a, V: Codec<Enc>, Enc: Encoding, K: KeySerde> DataStructure for Item<'a, V, Enc, K> {
    type Key = K;
    type Enc = Enc;
    type Value = V;
    type DsType = Terminal;

    fn with_prefix(prefix: Vec<u8>) -> Self {
        Self(KeyType::Raw(prefix), PhantomData)
    }

    fn should_skip_key(_: &Self::Key) -> bool {
        false
    }
}

impl<V: Codec<Enc>, Enc: Encoding> Item<'static, V, Enc> {
    pub const fn new(key: &'static [u8]) -> Self {
        Self(KeyType::Key(Cow::Borrowed(key)), PhantomData)
    }
}

impl<'a, V: Codec<Enc>, Enc: Encoding, K: KeySerde> Item<'a, V, Enc, K> {
    pub fn with_key(key: K) -> Self {
        Self(KeyType::Key(key), PhantomData)
    }

    pub fn may_load<S: Storage>(&self, storage: &S) -> Result<Option<V>, StorageError<Enc>> {
        let bytes = storage.get(&self.0)?;
        let value = bytes.map(|b| V::decode(b.as_slice())).transpose();
        value.map_err(StorageError::ValueDeserialize)
    }

    pub fn save<S: StorageMut>(&self, storage: &mut S, value: &V) -> Result<(), StorageError<Enc>> {
        let key = self.0.encode()?;
        let value = value.encode().map_err(StorageError::ValueSerialize)?;
        storage.set_raw(key, value);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use crate::mock::DisplayEncoding;

    use super::*;

    #[test]
    fn test_item() {
        const ITEM: Item<String, DisplayEncoding> = Item::new(b"foo");
        let item: Item<String, DisplayEncoding> = Item::with_key(Cow::Owned(b"bar".to_vec()));

        let mut storage: HashMap<Vec<u8>, Vec<u8>> = HashMap::new();
        assert_eq!(ITEM.may_load(&storage), Ok(None));
        assert_eq!(item.may_load(&storage), Ok(None));

        ITEM.save(&mut storage, &"baz".to_string()).unwrap();
        assert_eq!(ITEM.may_load(&storage), Ok(Some("baz".to_string())));
        assert_eq!(item.may_load(&storage), Ok(None));

        item.save(&mut storage, &"qux".to_string()).unwrap();
        assert_eq!(ITEM.may_load(&storage), Ok(Some("baz".to_string())));
        assert_eq!(item.may_load(&storage), Ok(Some("qux".to_string())));
    }
}
