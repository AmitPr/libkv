#[cfg(test)]
mod test {
    use std::collections::BTreeMap;
    use std::marker::PhantomData;

    use crate::containers::{Item, Map};
    use crate::mock::DisplayEncoding;

    #[test]
    fn compiles() {
        // type Map1 = Map<String, Item<String, DisplayEncoding>, DisplayEncoding>;
        // type Map2 = Map<
        //     String,
        //     Map<String, Item<String, DisplayEncoding>, DisplayEncoding>,
        //     DisplayEncoding,
        // >;
        // type Vec1 = Vector<Item<String, DisplayEncoding>, DisplayEncoding>;
        // type Vec2 = Vector<Vector<Item<String, DisplayEncoding>, DisplayEncoding>, DisplayEncoding>;
        // type VecMap =
        //     Vector<Map<String, Item<String, DisplayEncoding>, DisplayEncoding>, DisplayEncoding>;
        // type Item1 = Item<String, DisplayEncoding>;

        // println!("{}", std::any::type_name::<Map1>());
        // println!("{}", std::any::type_name::<<Map2 as Container>::FullKey>());
    }

    #[test]
    fn test_map() {
        // let mut storage: BTreeMap<Vec<u8>, Vec<u8>> = BTreeMap::new();
        // let map: Map<String, Item<(), String, DisplayEncoding>, DisplayEncoding> = Map(PhantomData);

        // let key = map.key("foo".to_string());

        // let key = CompoundKey::new("foo".to_string(), "bar".to_string());
        // assert_eq!(map.may_load(&storage, &key).unwrap(), None);
        // map.save(&mut storage, &key, &"baz".to_string()).unwrap();
        // assert_eq!(
        //     map.may_load(&storage, &key).unwrap(),
        //     Some("baz".to_string())
        // );

        // let key2 = CompoundKey::new("foo".to_string(), "baz".to_string());
        // assert_eq!(map.may_load(&storage, &key2).unwrap(), None);
        // map.save(&mut storage, &key2, &"qux".to_string()).unwrap();
        // assert_eq!(
        //     map.may_load(&storage, &key2).unwrap(),
        //     Some("qux".to_string())
        // );

        // println!("{:?}", storage);
    }
}
