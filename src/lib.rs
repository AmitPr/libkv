#![allow(unused)]

mod container;
mod containers;
mod iterator;
mod key;
mod serialization;
mod storage;

#[cfg(test)]
mod mock;
/*
So here's the deal:

Underlying KV stores are likely to give us two ways to iterate over KV pairs:
1. .keys() -> an iterator over just keys
2. .iter() -> an iterator over (key, value) pairs

These may work fine! We still have key information, and can use the key, by
stepping "backwards" through the nested containers to apply the correct
filtering rules.

However, if the underlying store also provides
3. .values() -> an iterator over just values

We're in trouble. We can't recover the associated key if we only have the value,
and we can't, thus, filter out inline metadata that containers may have.

Possible solutions:
1. Disallow .values()
2. Via traits, mark containers that do not have inline metadata, and only
   allow .values() on those containers.

For now? Let's just disallow .values()

Now, how does the iterator "structure" look like, from store to user invocation?

For this structure:
Container1<Container2<>>, where container 1 is wrapping container 2:

Store:       .iter(low, high)  -> (key, value) pairs
Container 1: .iter(store_iter) -> (maybe) filter certain pairs
Container 2: .iter(store_iter) -> (maybe) filter certain pairs (... etc.)

First: Notice that the store iterator should be wrapped from the "top down":
Container 1 gets the first opportunity to filter, then Container 2, etc.

Second: Container 1 may have (key, data) or (key, metadata) pairs fed into its
iterator. We can't just deserialize as <T = data>, so we can either:
1. Not deserialize at all, and have the iterator logic work solely via keys,
    treating the values as opaque bytes.
2. Do some fancy enum-based deserialization where we can detect metadata and
    data, matching the enum. Notice this solves (3) from above, since we don't
    use keys (and can thus iterate .values()), but we also run into the following
    issue: We could have different types of metadata that serialize differntly.
    e.g.: Container 1 may see (key, metadata1) and (key, metadata2) pairs, and
    will fail to deserialize metadata2 as it is Container 2's type.

So for now? I'm thinking (1), with some way to deserialize the data automatically
into the value type once we finish the last wrapping iterator.

*/
