use crate::{self as mirror_mirror, Enum};

use mirror_mirror::{EnumValue, FromReflect, GetField, Reflect};

#[test]
fn enum_value() {
    let mut enum_ = EnumValue::new("Foo")
        .with_field("foo", 1_i32)
        .with_field("bar", false);

    assert_eq!(enum_.get_field::<i32>("foo").unwrap(), &1);
    assert_eq!(enum_.get_field::<bool>("bar").unwrap(), &false);

    *enum_.get_field_mut("foo").unwrap() = 42;
    assert_eq!(enum_.get_field::<i32>("foo").unwrap(), &42);

    enum_.patch(&EnumValue::new("Foo").with_field("bar", true));
    assert_eq!(enum_.get_field::<bool>("bar").unwrap(), &true);

    let enum_ = EnumValue::from_reflect(&enum_).unwrap();
    assert_eq!(enum_.get_field::<i32>("foo").unwrap(), &42);
    assert_eq!(enum_.get_field::<bool>("bar").unwrap(), &true);

    let value = enum_.to_value();
    assert_eq!(value.get_field::<i32>("foo").unwrap(), &42);
    assert_eq!(value.get_field::<bool>("bar").unwrap(), &true);
    let mut has_fields = false;
    for (key, value) in value.as_enum().unwrap().fields() {
        has_fields = true;
        if key == "foo" {
            assert_eq!(value.downcast_ref::<i32>().unwrap(), &42);
        } else if key == "bar" {
            assert_eq!(value.downcast_ref::<bool>().unwrap(), &true);
        } else {
            panic!("unknown field: {key}");
        }
    }
    assert!(has_fields);
}

#[test]
fn static_enum() {
    #[derive(Reflect, Clone, Debug, PartialEq, Eq)]
    enum Foo {
        Foo { foo: i32, bar: bool },
        Bar { baz: String },
    }

    let mut enum_ = Foo::Foo { foo: 1, bar: false };

    assert_eq!(enum_.get_field::<i32>("foo").unwrap(), &1);
    assert_eq!(enum_.get_field::<bool>("bar").unwrap(), &false);

    *enum_.get_field_mut("foo").unwrap() = 42;
    assert_eq!(enum_.get_field::<i32>("foo").unwrap(), &42);

    enum_.patch(&EnumValue::new("Foo").with_field("bar", true));
    assert_eq!(enum_.get_field::<bool>("bar").unwrap(), &true);

    // variants with other names are ignored
    enum_.patch(&EnumValue::new("Ignored").with_field("bar", false));
    assert_eq!(enum_.get_field::<bool>("bar").unwrap(), &true);

    assert!(matches!(enum_, Foo::Foo { foo: 42, bar: true }));

    let enum_ = Foo::from_reflect(&enum_).unwrap();
    assert!(matches!(enum_, Foo::Foo { foo: 42, bar: true }));

    let value = enum_.to_value();
    assert_eq!(value.get_field::<i32>("foo").unwrap(), &42);
    assert_eq!(value.get_field::<bool>("bar").unwrap(), &true);
    let mut has_fields = false;
    for (key, value) in value.as_enum().unwrap().fields() {
        has_fields = true;
        if key == "foo" {
            assert_eq!(value.downcast_ref::<i32>().unwrap(), &42);
        } else if key == "bar" {
            assert_eq!(value.downcast_ref::<bool>().unwrap(), &true);
        } else {
            panic!("unknown field: {key}");
        }
    }
    assert!(has_fields);
}

#[test]
fn patching() {
    #[derive(Reflect, Clone, Debug, PartialEq, Eq)]
    enum Foo {
        A { a: i32 },
        B { b: bool },
    }

    // whole: static.patch(static)
    let mut foo = Foo::A { a: 1 };
    foo.patch(&Foo::B { b: false });
    assert!(matches!(dbg!(foo), Foo::B { b: false }));

    // part: static.patch(static)
    let mut foo = Foo::A { a: 1 };
    foo.patch(&Foo::A { a: 42 });
    assert!(matches!(dbg!(foo), Foo::A { a: 42 }));

    // whole: static.patch(value)
    let mut foo = Foo::A { a: 1 };
    foo.patch(&EnumValue::new("B").with_field("b", false));
    assert!(matches!(dbg!(foo), Foo::B { b: false }));

    // part: static.patch(value)
    let mut foo = Foo::A { a: 1 };
    foo.patch(&EnumValue::new("A").with_field("a", 42));
    assert!(matches!(dbg!(foo), Foo::A { a: 42 }));

    // whole: value.patch(static)
    let mut foo = EnumValue::new("A").with_field("a", 1);
    foo.patch(&Foo::B { b: false });
    assert_eq!(foo.get_field::<bool>("b").unwrap(), &false);

    // part: value.patch(static)
    let mut foo = EnumValue::new("A").with_field("a", 1);
    foo.patch(&Foo::A { a: 42 });
    assert_eq!(foo.get_field::<i32>("a").unwrap(), &42);

    // whole: value.patch(value)
    let mut foo = EnumValue::new("A").with_field("a", 1);
    foo.patch(&EnumValue::new("B").with_field("b", false));
    assert_eq!(foo.variant_name(), "B");
    assert!(foo.get_field::<i32>("a").is_none());
    assert_eq!(foo.get_field::<bool>("b").unwrap(), &false);

    // part: value.patch(value)
    let mut foo = EnumValue::new("A").with_field("a", 1);
    foo.patch(&EnumValue::new("A").with_field("a", 42));
    assert_eq!(foo.get_field::<i32>("a").unwrap(), &42);
}
