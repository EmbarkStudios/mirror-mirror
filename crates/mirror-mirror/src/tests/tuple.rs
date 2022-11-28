use crate::GetField;
use crate::Reflect;
use crate::TupleValue;

#[test]
fn tuple_value() {
    let mut tuple = TupleValue::new().with_element(1_i32).with_element(false);

    assert_eq!(tuple.get_field::<i32>(0).unwrap(), &1);
    assert_eq!(tuple.get_field::<bool>(1).unwrap(), &false);

    tuple.patch(&TupleValue::new().with_element(42_i32));
    assert_eq!(tuple.get_field::<i32>(0).unwrap(), &42);
    assert_eq!(tuple.get_field::<bool>(1).unwrap(), &false);
}

#[test]
fn static_tuple() {
    let mut tuple = (1_i32, false);

    assert_eq!(tuple.get_field::<i32>(0).unwrap(), &1);
    assert_eq!(tuple.get_field::<bool>(1).unwrap(), &false);

    tuple.patch(&TupleValue::new().with_element(42_i32));
    assert_eq!(tuple.get_field::<i32>(0).unwrap(), &42);
    assert_eq!(tuple.get_field::<bool>(1).unwrap(), &false);
}
