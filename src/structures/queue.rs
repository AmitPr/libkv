use crate::{
    Codec, Encoding, Item, IterableStorage, KeyEncoding, Map, Order, StorageError, StorageMut,
};
use std::ops::Bound;

// Single value per priority queue
pub struct PriorityQueue<'a, K: Codec<KeyEncoding> + Ord + Clone, V: Codec<Enc>, Enc: Encoding> {
    map: Map<'a, K, Item<'a, V, Enc>>,
}

impl<'a, K: Codec<KeyEncoding> + Ord + Clone, V: Codec<Enc>, Enc: Encoding>
    PriorityQueue<'a, K, V, Enc>
{
    pub const fn new(prefix: &'static [u8]) -> Self {
        Self {
            map: Map::new(prefix),
        }
    }

    pub fn push<S: StorageMut>(
        &self,
        storage: &mut S,
        priority: K,
        value: &V,
    ) -> Result<(), StorageError<Enc>> {
        self.map.at(priority)?.save(storage, value)
    }

    pub fn peek<S: IterableStorage>(
        &self,
        storage: &S,
        order: Order,
    ) -> Result<Option<(K, V)>, StorageError<Enc>> {
        let mut iter = self
            .map
            .range(storage, Bound::Unbounded, Bound::Unbounded, order)?;

        let item = iter.next();

        match item {
            Some(Ok((key, value))) => Ok(Some((key.0, value))),
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }

    pub fn pop<S: StorageMut + IterableStorage>(
        &self,
        storage: &mut S,
        order: Order,
    ) -> Result<Option<(K, V)>, StorageError<Enc>> {
        if let Some((key, value)) = self.peek(storage, order)? {
            self.map.at(key.clone())?.delete(storage)?;
            Ok(Some((key, value)))
        } else {
            Ok(None)
        }
    }
}

/*
// Multiple values per priority queue
pub struct MultiPriorityQueue<'a, K, V, Enc>
where
    K: KeySerde + Ord + Clone,
    Enc: Encoding,
    V: Codec<Enc>,
    usize: Codec<Enc>,
{
    map: Map<'a, K, Map<'a, usize, Item<'a, V, Enc>>>,
    counters: Map<'a, K, Item<'a, usize, Enc>>,
}

impl<'a, K, V, Enc> MultiPriorityQueue<'a, K, V, Enc>
where
    K: KeySerde + Ord + Clone,
    Enc: Encoding,
    V: Codec<Enc>,
    usize: Codec<Enc>,
{
    pub const fn new(prefix: &'static [u8]) -> Self {
        Self {
            map: Map::new(prefix),
            counters: Map::new(b"counters"),
        }
    }

    pub fn push<S: StorageMut>(
        &self,
        storage: &mut S,
        priority: K,
        value: &V,
    ) -> Result<(), StorageError<Enc>> {
        // Get or initialize counter
        let counter = self.counters.at(priority.clone())?;
        let next_id = counter.may_load(storage)?.unwrap_or(0) + 1;
        counter.save(storage, &next_id)?;

        // Save value
        self.map.at(priority)?.at(next_id)?.save(storage, value)
    }

    pub fn peek<S: IterableStorage>(
        &self,
        storage: &S,
        ascending: bool,
    ) -> Result<Option<(K, Vec<V>)>, StorageError<Enc>> {
        let iter = self
            .map
            .range(storage, Bound::Unbounded, Bound::Unbounded)?;

        let priority_map = if ascending {
            iter.into_iter().next()
        } else {
            iter.into_iter().last()
        };

        match priority_map {
            Some(Ok((key, _))) => {
                let values_iter = self.map.at(key.0.clone())?.range(
                    storage,
                    Bound::Unbounded,
                    Bound::Unbounded,
                )?;

                let mut values = Vec::new();
                for item in values_iter {
                    match item {
                        Ok((_, value)) => values.push(value),
                        Err(e) => return Err(e),
                    }
                }

                Ok(Some((key.0, values)))
            }
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }

    pub fn pop<S: StorageMut + IterableStorage>(
        &self,
        storage: &mut S,
        ascending: bool,
    ) -> Result<Option<(K, Vec<V>)>, StorageError<Enc>> {
        if let Some((priority, values)) = self.peek(storage, ascending)? {
            // Clear all values for this priority
            let priority_map = self.map.at(priority.clone())?;
            for i in 1..=values.len() as u64 {
                priority_map.at(i)?.save(storage, &values[i as usize - 1])?;
            }

            // Reset counter
            self.counters.at(priority.clone())?.save(storage, &0)?;

            Ok(Some((priority, values)))
        } else {
            Ok(None)
        }
    }
} */

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::DisplayEncoding;
    use std::collections::BTreeMap;

    #[test]
    fn test_priority_queue() {
        let mut storage = BTreeMap::new();
        let pq: PriorityQueue<i32, String, DisplayEncoding> = PriorityQueue::new(b"test_pq");

        // Test pushing elements
        pq.push(&mut storage, 3, &"third".to_string()).unwrap();
        pq.push(&mut storage, 1, &"first".to_string()).unwrap();
        pq.push(&mut storage, 2, &"second".to_string()).unwrap();

        // Test peeking
        assert_eq!(
            pq.peek(&storage, Order::Ascending).unwrap(),
            Some((1, "first".to_string()))
        );
        assert_eq!(
            pq.peek(&storage, Order::Descending).unwrap(),
            Some((3, "third".to_string()))
        );

        // Test popping
        assert_eq!(
            pq.pop(&mut storage, Order::Ascending).unwrap(),
            Some((1, "first".to_string()))
        );
        assert_eq!(
            pq.pop(&mut storage, Order::Ascending).unwrap(),
            Some((2, "second".to_string()))
        );
        assert_eq!(
            pq.pop(&mut storage, Order::Ascending).unwrap(),
            Some((3, "third".to_string()))
        );
        assert_eq!(pq.pop(&mut storage, Order::Ascending).unwrap(), None);
    }

    // #[test]
    // fn test_multi_priority_queue() {
    //     let mut storage = BTreeMap::new();
    //     let mpq: MultiPriorityQueue<i32, String, DisplayEncoding> =
    //         MultiPriorityQueue::new(b"test_mpq");

    //     // Test pushing elements
    //     mpq.push(&mut storage, 1, &"first-1".to_string()).unwrap();
    //     mpq.push(&mut storage, 1, &"first-2".to_string()).unwrap();
    //     mpq.push(&mut storage, 2, &"second-1".to_string()).unwrap();
    //     mpq.push(&mut storage, 2, &"second-2".to_string()).unwrap();

    //     // Test peeking
    //     let (priority, values) = mpq.peek(&storage, true).unwrap().unwrap();
    //     assert_eq!(priority, 1);
    //     assert_eq!(values, vec!["first-1".to_string(), "first-2".to_string()]);

    //     // Test popping
    //     let (priority, values) = mpq.pop(&mut storage, true).unwrap().unwrap();
    //     assert_eq!(priority, 1);
    //     assert_eq!(values, vec!["first-1".to_string(), "first-2".to_string()]);

    //     let (priority, values) = mpq.pop(&mut storage, false).unwrap().unwrap();
    //     assert_eq!(priority, 2);
    //     assert_eq!(values, vec!["second-1".to_string(), "second-2".to_string()]);

    //     assert_eq!(mpq.pop(&mut storage, true).unwrap(), None);
    // }
}
