use std::convert::Infallible;

use crate::{serialization::from_length_prefixed_bytes, CompoundKey, Key};

impl Key for () {
    type Error = Infallible;
    fn encode(&self) -> Vec<u8> {
        vec![]
    }

    fn decode(bytes: &mut &[u8]) -> Result<Self, Self::Error> {
        Ok(())
    }
}

impl Key for String {
    //TODO: Actually handle errors
    type Error = ();
    fn encode(&self) -> Vec<u8> {
        crate::serialization::to_length_prefixed_bytes(self)
    }

    fn decode(bytes: &mut &[u8]) -> Result<Self, Self::Error> {
        let data = from_length_prefixed_bytes(bytes).ok_or(())?;
        String::from_utf8(data).map_err(|_| ())
    }
}

impl Key for usize {
    // TODO: Error if decode not enough bytes.
    type Error = ();
    fn encode(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }

    fn decode(bytes: &mut &[u8]) -> Result<Self, Self::Error> {
        let (int_bytes, rest) = bytes.split_at(std::mem::size_of::<usize>());
        *bytes = rest;
        Ok(usize::from_be_bytes(int_bytes.try_into().unwrap()))
    }
}

macro_rules! compound_key_type {
    ($t:ident, $($r: ident),+) => {
        CompoundKey<$t, compound_key_type!($($r),+)>
    };
    ($t:ident) => {
        $t
    }
}

macro_rules! compound_key_into {
    ($f: tt, $($t:tt),+) => {
        CompoundKey::new($f, compound_key_into!($($t),+))
    };
    ($f: tt) => {
        $f
    }
}

macro_rules! impl_compound_key_from_tuple {
    ($l: ident $f: ident, $($v:ident $t:ident),+) => {
        impl_compound_key_from_tuple!($($v $t),+);
        impl<$f: Key, $($t:Key,)+> From<($f, $($t,)+)> for compound_key_type!($f, $($t),+)
        {
            fn from(($l, $($v,)+): ($f, $($t,)+)) -> Self {
                compound_key_into!($l, $($v),+)
            }
        }
    };

    ($l: ident $f: ident) => {}
}

impl_compound_key_from_tuple!(t T, u U, v V, w W, x X, y Y, z Z, a A, b B, c C, d D, e E);
