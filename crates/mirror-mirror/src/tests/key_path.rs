use alloc::collections::BTreeMap;

use crate::key_path;
use crate::key_path::*;
use crate::type_info::ScalarType;
use crate::type_info::TypeAtPath;
use crate::DescribeType;
use crate::Reflect;

#[test]
#[allow(clippy::bool_assert_comparison)]
fn works() {
    #[derive(Reflect, Clone, Debug)]
    #[reflect(crate_name(crate), opt_out(Default))] // TODO: implement default
    struct A {
        a: i32,
        b: B,
        c: C,
        d: BTreeMap<String, u32>,
        e: Vec<f32>,
    }

    #[derive(Reflect, Clone, Debug)]
    #[reflect(crate_name(crate), opt_out(Default))]
    struct B {
        c: bool,
    }

    #[derive(Reflect, Clone, Debug)]
    #[reflect(crate_name(crate), opt_out(Default))]
    enum C {
        C { d: String },
    }

    let mut a = A {
        a: 42,
        b: B { c: true },
        c: C::C {
            d: "foo".to_owned(),
        },
        d: BTreeMap::from([("fourtytwo".to_owned(), 42)]),
        e: Vec::from([1.0, 2.0, 3.0]),
    };

    assert!(a.get_at::<A>(&key_path!()).is_some());
    assert_eq!(a.get_at::<i32>(&key_path!(.a)).unwrap(), &42);
    assert_eq!(a.get_at::<bool>(&key_path!(.b.c)).unwrap(), &true);
    assert_eq!(a.get_at::<String>(&key_path!(.c::C.d)).unwrap(), &"foo");
    assert!(a.at(&key_path!(.c::DoesntExist)).is_none());
    assert_eq!(a.get_at::<u32>(&key_path!(.d["fourtytwo"])).unwrap(), &42);

    assert_eq!(a.get_at::<f32>(&key_path!(.e[0])).unwrap(), &1.0);
    assert_eq!(a.get_at::<f32>(&key_path!(.e[1])).unwrap(), &2.0);
    assert_eq!(a.get_at::<f32>(&key_path!(.e[2])).unwrap(), &3.0);
    assert!(a.at(&key_path!(.e[3])).is_none());

    assert_eq!(a.b.c, true);
    *a.get_at_mut(&key_path!(.b.c)).unwrap() = false;
    assert_eq!(a.b.c, false);
}

#[test]
fn display() {
    assert_eq!(
        key_path!(.a.0.b.c[1]["foo"]::D.e[3]).to_string(),
        ".a.0.b.c[1][\"foo\"]::D.e[3]"
    );
}

#[test]
fn query_type_info_struct() {
    #[derive(Reflect, Clone, Debug, Default)]
    #[reflect(crate_name(crate))]
    struct User {
        employer: Company,
    }

    #[derive(Reflect, Clone, Debug, Default)]
    #[reflect(crate_name(crate))]
    struct Company {
        countries: Vec<Country>,
    }

    #[derive(Reflect, Clone, Debug, Default)]
    #[reflect(crate_name(crate))]
    struct Country {
        name: String,
    }

    let user = User {
        employer: Company {
            countries: Vec::from([Country {
                name: "Denmark".to_owned(),
            }]),
        },
    };

    let key_path = key_path!(.employer.countries[0].name);

    assert_eq!(user.get_at::<String>(&key_path).unwrap(), "Denmark");

    let type_info = <User as DescribeType>::type_descriptor();

    assert!(matches!(
        dbg!(type_info.type_at(&key_path).unwrap()),
        TypeAtPath::Scalar(ScalarType::String)
    ));
}

#[test]
fn query_type_info_enum() {
    #[derive(Reflect, Clone, Debug)]
    #[reflect(crate_name(crate), opt_out(Default))]
    enum Foo {
        A { a: String },
        B(i32),
        C,
    }

    assert!(matches!(
        dbg!(<Foo as DescribeType>::type_descriptor()
            .type_at(&key_path!(::A.a))
            .unwrap()),
        TypeAtPath::Scalar(ScalarType::String)
    ));

    assert!(matches!(
        dbg!(<Foo as DescribeType>::type_descriptor()
            .type_at(&key_path!(::B.0))
            .unwrap()),
        TypeAtPath::Scalar(ScalarType::i32)
    ));

    assert!(<Foo as DescribeType>::type_descriptor()
        .type_at(&key_path!(::B[0]))
        .is_none());

    let info = <Foo as DescribeType>::type_descriptor();
    let variant = info.type_at(&key_path!(::C)).unwrap().as_variant().unwrap();

    assert_eq!(variant.name(), "C");
}

#[test]
fn select_tuple_field() {
    #[derive(Reflect, Clone, Debug, Default)]
    #[reflect(crate_name(crate))]
    struct Foo(i32, bool);

    let foo = Foo(42, true);

    assert_eq!(foo.get_at::<i32>(&key_path!(.0)).unwrap(), &42);
    assert_eq!(foo.get_at::<bool>(&key_path!(.1)).unwrap(), &true);
}

#[test]
fn breadcrumbs() {
    let path = key_path!(.a.b.c.d);

    let actual = path
        .breadcrumbs()
        .map(|key| key.iter().cloned().collect::<KeyPath>());

    let expected = Vec::from([
        key_path!(.a),
        key_path!(.a.b),
        key_path!(.a.b.c),
        key_path!(.a.b.c.d),
    ]);

    for (a, b) in actual.into_iter().zip(expected) {
        assert_eq!(a, b);
    }
}
