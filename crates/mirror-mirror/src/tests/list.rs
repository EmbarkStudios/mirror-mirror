use crate::FromReflect;
use crate::Reflect;

#[test]
fn indexing() {
    let list = Vec::from([1, 2, 3]);
    let list = list.reflect_ref().as_list().unwrap();

    assert_eq!(list.get(0).unwrap().downcast_ref::<i32>().unwrap(), &1);
    assert_eq!(list.get(1).unwrap().downcast_ref::<i32>().unwrap(), &2);
    assert_eq!(list.get(2).unwrap().downcast_ref::<i32>().unwrap(), &3);
    assert!(list.get(3).is_none());

    let value = list.to_value();
    let value = value.reflect_ref().as_list().unwrap();
    assert_eq!(value.get(0).unwrap().downcast_ref::<i32>().unwrap(), &1);
    assert_eq!(value.get(1).unwrap().downcast_ref::<i32>().unwrap(), &2);
    assert_eq!(value.get(2).unwrap().downcast_ref::<i32>().unwrap(), &3);
    assert!(value.get(3).is_none());

    let mut list = Vec::<i32>::from_reflect(list.as_reflect()).unwrap();
    assert_eq!(list, Vec::from([1, 2, 3]));

    list.patch(&Vec::from([42]));
    assert_eq!(list, Vec::from([42, 2, 3]));
}

#[test]
fn debug() {
    let list = Vec::from([1, 2, 3]);
    assert_eq!(format!("{:?}", list.as_reflect()), format!("{:?}", list));
    assert_eq!(format!("{:#?}", list.as_reflect()), format!("{:#?}", list));
}

#[test]
fn remove() {
    let mut list = Vec::from([1, 2, 3]);
    let list = list.as_list_mut().unwrap();
    assert_eq!(
        list.try_remove(2).unwrap().downcast_ref::<i32>().unwrap(),
        &3
    );
    assert!(list.try_remove(2).is_none());
    assert!(list.try_remove(1337).is_none());
}
