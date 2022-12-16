use crate::{FromReflect, Reflect, Typed};

#[test]
fn option_uses_none_as_default() {
    #[derive(Reflect, Clone, Debug, PartialEq, Eq)]
    #[reflect(crate_name(crate))]
    struct Foo {
        x: Option<i32>,
    }

    let default = <Foo as Typed>::type_descriptor().default_value().unwrap();

    let foo = Foo::from_reflect(&default).expect("`from_reflect` failed");
    assert_eq!(foo, Foo { x: None });
}
