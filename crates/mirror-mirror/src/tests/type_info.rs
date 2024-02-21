use core::any::type_name;
use core::hash::Hash;

use alloc::collections::BTreeMap;

use crate::key_path;
use crate::key_path::GetPath;
use crate::tuple_struct::TupleStructValue;
use crate::type_info::graph::OpaqueNode;
use crate::type_info::*;
use crate::FromReflect;
use crate::Reflect;
use crate::Value;

#[test]
fn struct_() {
    #[derive(Reflect, Clone, Debug, Default)]
    #[reflect(crate_name(crate))]
    struct Foo {
        n: i32,
        foos: Vec<Foo>,
    }

    let type_info = <Foo as DescribeType>::type_descriptor();

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
    #[derive(Reflect, Clone, Debug, Default, PartialEq, Eq)]
    #[reflect(crate_name(crate), meta(a = Foo(1337)))]
    struct Foo(i32);

    let type_info = <Foo as DescribeType>::type_descriptor();

    let foo = type_info.get_type().get_meta::<Foo>("a").unwrap();
    assert_eq!(foo, Foo(1337));
}

#[test]
fn type_to_root() {
    #[derive(Reflect, Clone, Debug, Default, PartialEq, Eq)]
    #[reflect(crate_name(crate), meta(a = Foo(1337)))]
    struct Foo(i32);

    let type_info = <Foo as DescribeType>::type_descriptor();

    assert_eq!(
        type_info.get_type().as_tuple_struct().unwrap().type_name(),
        type_name::<Foo>()
    );

    let type_info = type_info
        .get_type()
        .as_tuple_struct()
        .unwrap()
        .into_type_descriptor();
    assert_eq!(type_info.get_type().type_name(), type_name::<Foo>());
}

#[test]
fn two_types() {
    #[derive(Reflect, Clone, Debug, Default, PartialEq, Eq)]
    #[reflect(crate_name(crate))]
    struct Foo(i32);

    #[derive(Reflect, Clone, Debug, Default, PartialEq, Eq)]
    #[reflect(crate_name(crate))]
    struct Bar(bool);

    assert_eq!(
        <Foo as DescribeType>::type_descriptor()
            .as_tuple_struct()
            .unwrap()
            .field_type_at(0)
            .unwrap()
            .get_type()
            .as_scalar()
            .unwrap(),
        ScalarType::i32,
    );

    assert_eq!(
        <Bar as DescribeType>::type_descriptor()
            .as_tuple_struct()
            .unwrap()
            .field_type_at(0)
            .unwrap()
            .get_type()
            .as_scalar()
            .unwrap(),
        ScalarType::bool,
    )
}

#[test]
fn how_to_handle_generics() {
    #[derive(Reflect, Clone, Debug, PartialEq, Eq)]
    #[reflect(crate_name(crate), opt_out(Debug, Clone, Default))]
    struct Foo<T>(T)
    where
        T: Reflect + FromReflect + DescribeType;

    assert_eq!(
        <Foo<i32> as DescribeType>::type_descriptor()
            .as_tuple_struct()
            .unwrap()
            .field_type_at(0)
            .unwrap()
            .get_type()
            .as_scalar()
            .unwrap(),
        ScalarType::i32,
    );

    assert_eq!(
        <Foo<bool> as DescribeType>::type_descriptor()
            .as_tuple_struct()
            .unwrap()
            .field_type_at(0)
            .unwrap()
            .get_type()
            .as_scalar()
            .unwrap(),
        ScalarType::bool,
    );
}

#[test]
fn opaque_default() {
    struct Opaque(i32);

    impl DescribeType for Opaque {
        fn build(graph: &mut graph::TypeGraph) -> graph::NodeId {
            graph.get_or_build_node_with::<Self, _>(|graph| {
                OpaqueNode::new::<Self>(Default::default(), graph).default_value(Opaque(1337))
            })
        }
    }

    impl From<Opaque> for Value {
        fn from(opaque: Opaque) -> Self {
            let Opaque(n) = opaque;
            TupleStructValue::new().with_field(n).to_value()
        }
    }

    let type_descriptor = Opaque::type_descriptor();

    let default_value = type_descriptor.default_value().unwrap();

    assert_eq!(default_value.get_at::<i32>(&key_path!(.0)).unwrap(), &1337);
}

#[test]
fn basic_eq() {
    #[derive(Reflect, Clone, Debug, Default, PartialEq, Eq)]
    #[reflect(crate_name(crate))]
    struct Foo(i32);

    #[derive(Reflect, Clone, Debug, Default, PartialEq, Eq)]
    #[reflect(crate_name(crate))]
    struct Bar {
        b: bool,
    }

    assert_eq!(
        <Foo as DescribeType>::type_descriptor(),
        <Foo as DescribeType>::type_descriptor(),
    );

    assert_eq!(
        <Bar as DescribeType>::type_descriptor(),
        <Bar as DescribeType>::type_descriptor(),
    );

    assert_ne!(
        <Foo as DescribeType>::type_descriptor(),
        <Bar as DescribeType>::type_descriptor(),
    );
}

#[test]
fn basic_hash() {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    #[derive(Reflect, Clone, Debug, Default, PartialEq, Eq)]
    #[reflect(crate_name(crate))]
    struct Foo {
        a: i32,
    }

    #[derive(Reflect, Clone, Debug, Default, PartialEq, Eq)]
    #[reflect(crate_name(crate))]
    struct Bar {
        b: bool,
    }

    let s = RandomState::new();

    let mut hasher = s.build_hasher();
    <Foo as DescribeType>::type_descriptor().hash(&mut hasher);
    let foo_hash = hasher.finish();

    let mut hasher = s.build_hasher();
    <Bar as DescribeType>::type_descriptor().hash(&mut hasher);
    let bar_hash = hasher.finish();

    assert_ne!(foo_hash, bar_hash);

    let mut hasher = s.build_hasher();
    <Foo as DescribeType>::type_descriptor().hash(&mut hasher);
    let foo_hash_2 = hasher.finish();

    let mut hasher = s.build_hasher();
    <Bar as DescribeType>::type_descriptor().hash(&mut hasher);
    let bar_hash_2 = hasher.finish();

    assert_eq!(foo_hash, foo_hash_2);
    assert_eq!(bar_hash, bar_hash_2);
}

#[test]
fn has_default_value() {
    #[derive(Reflect, Clone, Debug, Default)]
    #[reflect(crate_name(crate))]
    struct A {
        a: String,
    }

    #[derive(Reflect, Clone, Debug, Default)]
    #[reflect(crate_name(crate))]
    struct B(String);

    #[derive(Reflect, Clone, Debug)]
    #[reflect(crate_name(crate))]
    enum C {
        C(i32),
    }

    assert!(<A as DescribeType>::type_descriptor().has_default_value());
    assert!(<B as DescribeType>::type_descriptor().has_default_value());
    assert!(<C as DescribeType>::type_descriptor().has_default_value());
    assert!(<(i32, String) as DescribeType>::type_descriptor().has_default_value());
    assert!(<[i32; 3] as DescribeType>::type_descriptor().has_default_value());
    assert!(<BTreeMap<String, i32> as DescribeType>::type_descriptor().has_default_value());
    assert!(<i32 as DescribeType>::type_descriptor().has_default_value());

    // value doesn't have a default
    assert!(!<[Value; 3] as DescribeType>::type_descriptor().has_default_value());
    assert!(!<Value as DescribeType>::type_descriptor().has_default_value());
}
