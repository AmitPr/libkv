use std::{marker::PhantomData, ops::Bound};

use disjoint_impls::disjoint_impls;

use crate::v2::container::{Branch, Leaf, PartialToInner};

use super::{
    container::Container,
    key::{DecodeResult, EncodeResult, Key, KeySerde},
    storage::{IterableStorage, RawKey},
};

pub struct ContainerIter<'a, C: Container + ?Sized> {
    iter: super::storage::Iter<'a, (RawKey<C>, Vec<u8>)>,
    _marker: PhantomData<C>,
}

impl<'a, C: IterSkip + Iterable> Iterator for ContainerIter<'a, C> {
    type Item = DecodeResult<(<C::FullKey as KeySerde>::PartialKey, C::Value), C::FullKey>;

    fn next(&mut self) -> Option<Self::Item> {
        /*
           1. Grab item from storage's iterator
           2. Deserialize (potentially partial) key
           3. Ask container type to check if this key should be skipped (recurses into children)
           4. If it should be skipped, go to 1.
           6. Otherwise, return the full key and value.
        */
        loop {
            // Grab from storage iterator
            let (raw, val) = self.iter.next()?;
            // Deserialize (potentially partial) key
            let bytes = &mut raw.0.as_slice();
            let decoded = C::FullKey::partial_decode(bytes);
            match decoded {
                Err(err) => return Some(Err(err)),
                Ok(Some(key)) => {
                    if C::_do_should_skip(&key) {
                        continue;
                    }

                    let val = todo!("Deserialize value, propagate properly.");
                    return Some(Ok((key, val)));
                }
                Ok(None) => {
                    // A partial key that even the root container can't deserialize.
                    todo!("Figure out what to do in this case.")
                }
            }
            // let cur_deser = raw.
        }
        todo!()
    }
}

disjoint_impls! {
    trait IterSkip: PartialToInner + Iterable {
        fn _do_should_skip(key: &<Self::FullKey as KeySerde>::PartialKey) -> bool {
            false
        }
    }

    impl<
            C: PartialToInner<ChildKey = CK::PartialKey>
                + Iterable
                + Container<ContainerType = Branch<Inner>>,
            Inner: IterSkip + Container<FullKey = CK>,
            CK: KeySerde,
        > IterSkip for C
    {
        fn _do_should_skip(key: &<Self::FullKey as KeySerde>::PartialKey) -> bool {
            if Self::should_skip(key) {
                return true;
            }

            Self::partial_to_inner(key)
                .map(|child_key| Inner::_do_should_skip(child_key))
                .unwrap_or(false)
        }
    }

    impl<C: PartialToInner + Iterable + Container<ContainerType=Leaf<V>>, V> IterSkip for C {
        fn _do_should_skip(_key: &<Self::FullKey as KeySerde>::PartialKey) -> bool {
            false
        }
    }
}

pub trait Iterable: Container<FullKey: KeySerde<PartialKey: Key<Container = Self>>> {
    // /// As containers may be nested, and may have metadata that shouldn't be
    // /// exposed by a "raw iteration". This function can be overridden to wrap
    // /// an underlying iterator with one that may filter certain elements.
    // fn iter(&self) -> Box<dyn Iterator<Item = Self::Key>>;

    fn should_skip(key: &<Self::FullKey as KeySerde>::PartialKey) -> bool {
        false
    }

    fn range<'a, S: IterableStorage>(
        &'a self,
        storage: &'a S,
        low: Bound<<Self::FullKey as KeySerde>::PartialKey>,
        high: Bound<<Self::FullKey as KeySerde>::PartialKey>,
    ) -> EncodeResult<ContainerIter<Self>, <Self::FullKey as KeySerde>::PartialKey> {
        let base = storage.iter(low, high)?;
        let iter = ContainerIter {
            iter: base,
            _marker: PhantomData,
        };

        Ok(iter)
    }
}
