use alloc::vec::Vec;

use crate::struct_::TupleStructValue;
use crate::FromReflect;
use crate::GetField;
use crate::Reflect;
use crate::TupleStruct;

#[test]
fn tuple_value() {
    let mut tuple = TupleStructValue::new().with_field(1_i32).with_field(false);

    assert_eq!(tuple.get_field::<i32>(0).unwrap(), &1);
    assert_eq!(tuple.get_field::<bool>(1).unwrap(), &false);

    tuple.patch(&TupleStructValue::new().with_field(42_i32));
    assert_eq!(tuple.get_field::<i32>(0).unwrap(), &42);
    assert_eq!(tuple.get_field::<bool>(1).unwrap(), &false);
}

#[test]
fn static_tuple() {
    #[derive(Reflect, Default, Clone, Eq, PartialEq, Debug)]
    #[reflect(crate_name(crate))]
    struct A(i32, bool);

    let mut tuple = A(1_i32, false);

    assert_eq!(tuple.get_field::<i32>(0).unwrap(), &1);
    assert_eq!(tuple.get_field::<bool>(1).unwrap(), &false);

    tuple.patch(&TupleStructValue::new().with_field(42_i32));
    assert_eq!(tuple.get_field::<i32>(0).unwrap(), &42);
    assert_eq!(tuple.get_field::<bool>(1).unwrap(), &false);

    let mut tuple = A::from_reflect(&tuple.to_value()).unwrap();
    assert!(matches!(tuple, A(42, false)));

    let fields = tuple.fields().collect::<Vec<_>>();
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0].downcast_ref::<i32>().unwrap(), &42);
    assert_eq!(fields[1].downcast_ref::<bool>().unwrap(), &false);

    tuple.field_mut(1).unwrap().patch(&true);
    assert!(tuple.1);
}

#[test]
fn from_reflect_with_value() {
    #[derive(Debug, Clone, Reflect, Default)]
    #[reflect(crate_name(crate))]
    pub struct Foo(Number);

    #[derive(Debug, Clone, Reflect, Default)]
    #[reflect(crate_name(crate))]
    pub enum Number {
        #[default]
        One,
        Two,
        Three,
    }

    let value = TupleStructValue::new().with_field(Number::One);

    assert!(Foo::from_reflect(&value).is_some());
}
