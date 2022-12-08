use core::any::type_name;

use crate::key_path;
use crate::key_path::GetTypePath;
use crate::type_info::ScalarType;
use crate::Reflect;

#[test]
fn typed_value() {
    #[derive(Reflect, Debug, Clone)]
    #[reflect(crate_name(crate))]
    enum Foo {
        A { x: X },
        B(String),
    }

    #[derive(Reflect, Debug, Clone)]
    #[reflect(crate_name(crate))]
    struct X(i32);

    let value = Foo::A { x: X(123) }.to_typed_value();

    assert_eq!(value.type_info().type_name(), type_name::<Foo>());

    assert_eq!(
        value
            .type_info()
            .as_enum()
            .unwrap()
            .variants()
            .map(|v| v.name())
            .collect::<Vec<_>>(),
        vec!["A", "B"],
    );

    assert!(matches!(
        value
            .type_info()
            .at_type(&key_path!({ A }.x[0]))
            .unwrap()
            .as_scalar()
            .unwrap(),
        ScalarType::i32
    ));
}
