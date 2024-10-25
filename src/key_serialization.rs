use super::{KeyDeserializeError, KeySerializeError};

pub trait KeySerde: Sized {
    fn encode(&self) -> Result<Vec<u8>, KeySerializeError>;
    fn decode(bytes: &mut &[u8]) -> Result<Self, KeyDeserializeError>;
}

impl KeySerde for usize {
    fn encode(&self) -> Result<Vec<u8>, KeySerializeError> {
        Ok(self.to_be_bytes().to_vec())
    }

    fn decode(bytes: &mut &[u8]) -> Result<Self, KeyDeserializeError> {
        type IntBytes = [u8; std::mem::size_of::<usize>()];
        let (int_bytes, rest): (&IntBytes, &[u8]) =
            bytes
                .split_first_chunk()
                .ok_or(KeyDeserializeError::NotEnoughBytes(
                    std::mem::size_of::<usize>(),
                    bytes.len(),
                ))?;
        *bytes = rest;
        Ok(usize::from_be_bytes(*int_bytes))
    }
}

impl KeySerde for () {
    fn encode(&self) -> Result<Vec<u8>, KeySerializeError> {
        Ok(vec![])
    }

    fn decode(_: &mut &[u8]) -> Result<Self, KeyDeserializeError> {
        Ok(())
    }
}

impl KeySerde for String {
    fn encode(&self) -> Result<Vec<u8>, KeySerializeError> {
        let mut encoded = Vec::with_capacity(self.len() + std::mem::size_of::<usize>());
        encoded.extend(self.len().encode()?);
        encoded.extend(self.as_bytes());
        Ok(encoded)
    }

    fn decode(bytes: &mut &[u8]) -> Result<Self, KeyDeserializeError> {
        let len = usize::decode(bytes)?;
        let (data, rest) = bytes
            .split_at_checked(len)
            .ok_or(KeyDeserializeError::NotEnoughBytes(len, bytes.len()))?;
        *bytes = rest;
        String::from_utf8(data.to_vec()).map_err(Into::into)
    }
}

impl KeySerde for std::borrow::Cow<'_, [u8]> {
    fn encode(&self) -> Result<Vec<u8>, KeySerializeError> {
        let mut encoded = Vec::with_capacity(self.len() + std::mem::size_of::<usize>());
        encoded.extend(self.len().encode()?);
        encoded.extend(self.iter());
        Ok(encoded)
    }

    fn decode(bytes: &mut &[u8]) -> Result<Self, KeyDeserializeError> {
        let len = usize::decode(bytes)?;
        let (data, rest) = bytes
            .split_at_checked(len)
            .ok_or(KeyDeserializeError::NotEnoughBytes(len, bytes.len()))?;
        *bytes = rest;
        Ok(std::borrow::Cow::Owned(data.to_vec()))
    }
}

impl KeySerde for Vec<u8> {
    fn encode(&self) -> Result<Vec<u8>, KeySerializeError> {
        let len = self.len();
        let mut encoded = Vec::with_capacity(len + std::mem::size_of::<usize>());
        encoded.extend(len.encode()?);
        encoded.extend(self);
        Ok(encoded)
    }

    fn decode(bytes: &mut &[u8]) -> Result<Self, KeyDeserializeError> {
        let len = usize::decode(bytes)?;
        let (data, rest) = bytes
            .split_at_checked(len)
            .ok_or(KeyDeserializeError::NotEnoughBytes(len, bytes.len()))?;
        *bytes = rest;
        Ok(data.to_vec())
    }
}

impl<K: KeySerde> KeySerde for Option<K> {
    fn encode(&self) -> Result<Vec<u8>, KeySerializeError> {
        match self {
            Some(k) => k.encode(),
            None => Ok(vec![]),
        }
    }

    fn decode(bytes: &mut &[u8]) -> Result<Self, KeyDeserializeError> {
        if bytes.is_empty() {
            return Ok(None);
        }
        K::decode(bytes).map(Some)
    }
}

macro_rules! impl_tuple_keyserde {
    ($l: ident $f: ident, $($v:ident $t:ident),+) => {
        impl_tuple_keyserde!($($v $t),+);
        impl<$f: KeySerde, $($t:KeySerde,)+> KeySerde for ($f, $($t,)+)
        {
            fn encode(&self) -> Result<Vec<u8>, KeySerializeError> {

                let ($l, $($v,)+) = self;
                let mut encoded = Vec::new();
                encoded.extend($l.encode()?);
                $(
                    encoded.extend($v.encode()?);
                )+
                Ok(encoded)
            }

            fn decode(bytes: &mut &[u8]) -> Result<Self, KeyDeserializeError> {
                let $l = $f::decode(bytes)?;
                $(
                    let $v = $t::decode(bytes)?;
                )+
                Ok(($l, $($v,)+))
            }
        }
    };

    ($l: ident $f: ident) => {}
}

impl_tuple_keyserde!(t T, u U, v V, w W, x X, y Y, z Z, a A, b B, c C, d D, e E);
