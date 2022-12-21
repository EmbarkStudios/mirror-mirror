use alloc::collections::BTreeMap;

use crate::key_path;
use crate::key_path::GetPath;
use crate::GetField;
use crate::GetFieldMut;
use crate::Map;
use crate::Reflect;
use crate::Typed;

#[test]
fn works() {
    let mut map = BTreeMap::from([(1, 1)]);
    let map = map.as_reflect_mut().as_map_mut().unwrap();

    assert_eq!(map.get(&1).unwrap().downcast_ref::<i32>().unwrap(), &1);
    assert_eq!(map.get_field::<i32>(1_i32).unwrap(), &1);
    assert_eq!(map.get_field_mut::<i32>(1_i32).unwrap(), &mut 1);

    let map = BTreeMap::from([("foo".to_owned(), 1)]);
    let map = map.as_reflect().as_map().unwrap();
    assert_eq!(
        map.get(&"foo".to_owned())
            .unwrap()
            .downcast_ref::<i32>()
            .unwrap(),
        &1
    );
    assert_eq!(map.get_field::<i32>("foo").unwrap(), &1);
}

#[test]
fn exotic_key_type() {
    #[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Reflect)]
    #[reflect(crate_name(crate))]
    struct Foo(i32);

    let map = BTreeMap::from([(Foo(1), 1), (Foo(2), 2)]);
    let map: &dyn Map = map.as_map().unwrap();

    assert_eq!(map.get(&Foo(1)).unwrap().downcast_ref::<i32>().unwrap(), &1);
    assert_eq!(map.get(&Foo(2)).unwrap().downcast_ref::<i32>().unwrap(), &2);
    assert!(map.get(&Foo(3)).is_none());

    assert_eq!(map.get_at::<i32>(&key_path!([Foo(1)])).unwrap(), &1);
    assert_eq!(map.get_at::<i32>(&key_path!([Foo(2)])).unwrap(), &2);
    assert!(map.get_at::<i32>(&key_path!([Foo(3)])).is_none());
}

#[test]
fn exoctic_value_type() {
    #[derive(Debug, Clone, Reflect)]
    #[reflect(crate_name(crate))]
    struct Foo {
        array: [i32; 5],
        tuple: (Vec<i32>, bool),
    }

    let mut map = BTreeMap::<i32, Foo>::new();
    let foo_default_value = <Foo as Typed>::type_descriptor().default_value().unwrap();
    map.as_map_mut().unwrap().insert(&1, &foo_default_value);
    assert_eq!(map.len(), 1);
}
