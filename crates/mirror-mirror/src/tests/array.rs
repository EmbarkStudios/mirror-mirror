use crate::FromReflect;
use crate::Reflect;
use crate::DescribeType;

#[test]
fn from_default() {
    #[derive(Debug, Clone, Reflect, PartialEq)]
    #[reflect(crate_name(crate))]
    struct Foo([i32; 5]);

    let foo_default_value = <Foo as DescribeType>::type_descriptor().default_value().unwrap();

    let foo = Foo::from_reflect(&foo_default_value).unwrap();

    assert_eq!(foo, Foo([0, 0, 0, 0, 0]))
}
