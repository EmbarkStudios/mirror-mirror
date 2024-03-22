use std::collections::HashMap;

use crate::{DescribeType, FromReflect, Reflect};

#[test]
fn option_uses_none_as_default() {
    #[derive(Reflect, Clone, Debug, Default, PartialEq, Eq)]
    #[reflect(crate_name(crate))]
    struct Foo {
        x: Option<i32>,
    }

    let default = <Foo as DescribeType>::type_descriptor()
        .default_value()
        .unwrap();

    let foo = Foo::from_reflect(&default).expect("`from_reflect` failed");
    assert_eq!(foo, Foo { x: None });
}

#[test]
fn hash() {
    let map = HashMap::from([
        (1_i32.to_value(), "one"),
        ("foo".to_owned().to_value(), "two"),
    ]);

    assert_eq!(map.get(&1_i32.to_value()).unwrap(), &"one");
    assert_eq!(map.get(&"foo".to_owned().to_value()).unwrap(), &"two");
    assert!(map.get(&true.to_value()).is_none());
}
