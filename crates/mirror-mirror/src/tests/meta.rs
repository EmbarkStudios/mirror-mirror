use crate::type_info::GetMeta;
use crate::Reflect;
use crate::Typed;

#[test]
fn works() {
    #[derive(Reflect, Debug, Clone)]
    #[reflect(crate_name(crate), meta(foo = "bar", baz = 42))]
    struct Foo;

    let type_info = <Foo as Typed>::type_info();
    let type_info = type_info.get_type().as_struct().unwrap();

    assert_eq!(
        type_info
            .meta("foo")
            .unwrap()
            .downcast_ref::<String>()
            .unwrap(),
        "bar"
    );

    assert_eq!(
        type_info
            .meta("baz")
            .unwrap()
            .downcast_ref::<i32>()
            .unwrap(),
        &42,
    );
}

#[derive(Reflect, Debug, Clone)]
#[reflect(crate_name(crate), meta(n = 1))]
struct A {
    #[reflect(meta(n = 1))]
    a: String,
}

#[derive(Reflect, Debug, Clone)]
#[reflect(crate_name(crate), meta(n = 1))]
struct B(#[reflect(meta(n = 1))] String);

#[derive(Reflect, Debug, Clone)]
#[reflect(crate_name(crate), meta(n = 1))]
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
