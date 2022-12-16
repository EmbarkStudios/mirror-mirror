use core::any::type_name;

use crate::type_info::*;
use crate::Reflect;

#[test]
fn struct_() {
    #[derive(Reflect, Clone, Debug)]
    #[reflect(crate_name(crate))]
    struct Foo {
        n: i32,
        foos: Vec<Foo>,
    }

    let type_info = <Foo as Typed>::type_descriptor();

    assert_eq!(
        type_info.get_type().type_name(),
        "mirror_mirror::tests::type_info::struct_::Foo"
    );

    let struct_ = type_info.get_type().as_struct().unwrap();

    assert_eq!(
        struct_.type_name(),
        "mirror_mirror::tests::type_info::struct_::Foo"
    );

    for field in struct_.field_types() {
        match field.name() {
            "foos" => {
                assert_eq!(
                    field.get_type().type_name(),
                    "alloc::vec::Vec<mirror_mirror::tests::type_info::struct_::Foo>"
                );

                let list = field.get_type().as_list().unwrap();

                assert_eq!(
                    list.type_name(),
                    "alloc::vec::Vec<mirror_mirror::tests::type_info::struct_::Foo>"
                );

                assert_eq!(
                    list.element_type().type_name(),
                    "mirror_mirror::tests::type_info::struct_::Foo"
                );
            }
            "n" => {
                assert_eq!(field.get_type().type_name(), "i32");
                let scalar = field.get_type().as_scalar().unwrap();
                assert_eq!(scalar.type_name(), "i32");
            }
            _ => panic!("wat"),
        }
    }
}

#[test]
fn enum_() {
    #[derive(Reflect, Clone, Debug)]
    #[reflect(crate_name(crate))]
    enum Foo {
        A { a: String },
        B(Vec<Foo>),
        C,
    }
}

#[test]
fn complex_meta_type() {
    #[derive(Reflect, Clone, Debug, PartialEq, Eq)]
    #[reflect(crate_name(crate), meta(a = Foo(1337)))]
    struct Foo(i32);

    let type_info = <Foo as Typed>::type_descriptor();

    let foo = type_info.get_type().get_meta::<Foo>("a").unwrap();
    assert_eq!(foo, Foo(1337));
}

#[test]
fn type_to_root() {
    #[derive(Reflect, Clone, Debug, PartialEq, Eq)]
    #[reflect(crate_name(crate), meta(a = Foo(1337)))]
    struct Foo(i32);

    let type_info = <Foo as Typed>::type_descriptor();

    assert_eq!(
        type_info.get_type().as_tuple_struct().unwrap().type_name(),
        type_name::<Foo>()
    );

    let type_info = type_info
        .get_type()
        .as_tuple_struct()
        .unwrap()
        .to_type_root();
    assert_eq!(type_info.get_type().type_name(), type_name::<Foo>());
}
