/// Private traits for sealed pattern.

pub trait Sealed {}

macro_rules! seal {
    ($t:ident) => {
        impl Sealed for $t {}
    };
}

pub(crate) use seal;
