use crate::{CompoundKey, FixedSize, Key};

impl Key for () {
    type Size = FixedSize<0>;

    fn encode(&self) -> Vec<u8> {
        vec![]
    }

    fn decode(bytes: &mut &[u8]) -> Self {}
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
        CompoundKey($f, compound_key_into!($($t),+))
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
