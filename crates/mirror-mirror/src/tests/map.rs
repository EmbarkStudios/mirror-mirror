use crate::GetField;
use crate::GetFieldMut;
use crate::Reflect;
use std::collections::BTreeMap;

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
