use crate::{self as mirror_mirror, FromReflect, GetField, Reflect, TupleStruct, TupleStructValue};

#[test]
fn tuple_value() {
    let mut tuple = TupleStructValue::new()
        .with_element(1_i32)
        .with_element(false);

    assert_eq!(tuple.get_field::<i32>(0).unwrap(), &1);
    assert_eq!(tuple.get_field::<bool>(1).unwrap(), &false);

    tuple.patch(&TupleStructValue::new().with_element(42_i32));
    assert_eq!(tuple.get_field::<i32>(0).unwrap(), &42);
    assert_eq!(tuple.get_field::<bool>(1).unwrap(), &false);
}

#[test]
fn static_tuple() {
    #[derive(Reflect, Default, Clone, Eq, PartialEq, Debug)]
    struct A(i32, bool);

    let mut tuple = A(1_i32, false);

    assert_eq!(tuple.get_field::<i32>(0).unwrap(), &1);
    assert_eq!(tuple.get_field::<bool>(1).unwrap(), &false);

    tuple.patch(&TupleStructValue::new().with_element(42_i32));
    assert_eq!(tuple.get_field::<i32>(0).unwrap(), &42);
    assert_eq!(tuple.get_field::<bool>(1).unwrap(), &false);

    let mut tuple = A::from_reflect(&tuple.to_value()).unwrap();
    assert!(matches!(tuple, A(42, false)));

    let elements = tuple.elements().collect::<Vec<_>>();
    assert_eq!(elements.len(), 2);
    assert_eq!(elements[0].downcast_ref::<i32>().unwrap(), &42);
    assert_eq!(elements[1].downcast_ref::<bool>().unwrap(), &false);

    tuple.element_mut(1).unwrap().patch(&true);
    assert!(tuple.1);
}
