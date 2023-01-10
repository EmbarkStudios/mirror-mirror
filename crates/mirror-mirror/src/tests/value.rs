use crate::{FromReflect, Reflect, DescribeType};

#[test]
fn option_uses_none_as_default() {
    #[derive(Reflect, Clone, Debug, PartialEq, Eq)]
    #[reflect(crate_name(crate))]
    struct Foo {
        x: Option<i32>,
    }

    let default = <Foo as DescribeType>::type_descriptor().default_value().unwrap();

    let foo = Foo::from_reflect(&default).expect("`from_reflect` failed");
    assert_eq!(foo, Foo { x: None });
}
