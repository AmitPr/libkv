pub trait Key {
    type Error;

    fn encode(&self) -> Vec<u8>;

    /// Decode a key from a byte slice, consuming the bytes necessary to decode the key.
    fn decode(bytes: &mut &[u8]) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

#[derive(Debug)]
pub struct CompoundKey<T: Key, U: Key>(T, U);

impl<K: Key> CompoundKey<K, ()> {
    pub fn new_prefix(key: K) -> Self {
        CompoundKey(key, ())
    }
}

impl<T: Key, U: Key> CompoundKey<T, U> {
    pub fn new(prefix: T, suffix: U) -> Self {
        CompoundKey(prefix, suffix)
    }

    pub fn prefix(&self) -> &T {
        &self.0
    }

    pub fn suffix(&self) -> &U {
        &self.1
    }

    pub fn into_inner(self) -> (T, U) {
        (self.0, self.1)
    }
}

// TODO: This would be nice, but need an impl that "auto-flattens" the type
// impl<K: Key> CompoundKey<(), K> {
//     pub fn flatten(self) -> K {
//         self.1
//     }
// }

impl<T: Key, U: Key> Key for CompoundKey<T, U> {
    type Error = (Option<T::Error>, Option<U::Error>);
    fn encode(&self) -> Vec<u8> {
        let mut bytes = self.0.encode();
        bytes.extend_from_slice(&self.1.encode());
        bytes
    }

    fn decode(bytes: &mut &[u8]) -> Result<Self, Self::Error> {
        let key = T::decode(bytes).map_err(|e| (Some(e), None))?;
        let unit = U::decode(bytes).map_err(|e| (None, Some(e)))?;
        Ok(CompoundKey(key, unit))
    }
}
