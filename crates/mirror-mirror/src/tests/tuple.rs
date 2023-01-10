use crate::tuple::TupleValue;
use crate::DescribeType;
use crate::FromReflect;
use crate::GetField;
use crate::Reflect;

#[test]
fn tuple_value() {
    let mut tuple = TupleValue::new().with_field(1_i32).with_field(false);

    assert_eq!(tuple.get_field::<i32>(0).unwrap(), &1);
    assert_eq!(tuple.get_field::<bool>(1).unwrap(), &false);

    tuple.patch(&TupleValue::new().with_field(42_i32));
    assert_eq!(tuple.get_field::<i32>(0).unwrap(), &42);
    assert_eq!(tuple.get_field::<bool>(1).unwrap(), &false);
}

#[test]
fn static_tuple() {
    let mut tuple = (1_i32, false);

    assert_eq!(tuple.get_field::<i32>(0).unwrap(), &1);
    assert_eq!(tuple.get_field::<bool>(1).unwrap(), &false);

    tuple.patch(&TupleValue::new().with_field(42_i32));
    assert_eq!(tuple.get_field::<i32>(0).unwrap(), &42);
    assert_eq!(tuple.get_field::<bool>(1).unwrap(), &false);
}

#[test]
fn from_default() {
    type Pair = (i32, bool);

    let default_value = <Pair as DescribeType>::type_descriptor()
        .default_value()
        .unwrap();

    let foo = Pair::from_reflect(&default_value).unwrap();

    assert_eq!(foo, (0, false));
}
