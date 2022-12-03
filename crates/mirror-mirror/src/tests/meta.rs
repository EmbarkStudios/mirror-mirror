use crate as mirror_mirror;
use crate::Reflect;
use crate::Typed;
use mirror_mirror::type_info::TypeInfo;

#[test]
fn works() {
    #[derive(Reflect, Debug, Clone)]
    #[reflect(meta(foo = "bar", baz = 42))]
    struct Foo;

    let type_info = <Foo as Typed>::type_info();
    let type_info = match type_info.type_() {
        TypeInfo::Struct(Some(type_info)) => type_info,
        other => panic!("expected struct got {other:?}"),
    };

    assert_eq!(
        type_info
            .get_meta("foo")
            .unwrap()
            .downcast_ref::<String>()
            .unwrap(),
        "bar"
    );

    assert_eq!(
        type_info
            .get_meta("baz")
            .unwrap()
            .downcast_ref::<i32>()
            .unwrap(),
        &42,
    );
}

#[derive(Reflect, Debug, Clone)]
#[reflect(meta(n = 1))]
struct A {
    #[reflect(meta(n = 1))]
    a: String,
}

#[derive(Reflect, Debug, Clone)]
#[reflect(meta(n = 1))]
struct B(#[reflect(meta(n = 1))] String);

#[derive(Reflect, Debug, Clone)]
#[reflect(meta(n = 1))]
enum C {
    #[reflect(meta(n = 1))]
    A {
        #[reflect(meta(n = 1))]
        a: String,
    },

    #[reflect(meta(n = 1))]
    B(#[reflect(meta(n = 1))] String),

    #[reflect(meta(n = 1))]
    C,
}
