use crate as mirror_mirror;

use mirror_mirror::{
    enum_::{EnumFieldsIter, EnumFieldsIterMut},
    EnumValue, FromReflect, GetField, Reflect,
};

#[derive(Reflect, Clone, Debug, PartialEq, Eq)]
enum Foo {
    Foo { foo: i32, bar: bool },
    Bar { baz: String },
}

#[test]
fn enum_value() {
    let mut enum_ = EnumValue::new("Foo")
        .with_field("foo", 1_i32)
        .with_field("bar", false);

    assert_eq!(enum_.get_field::<i32>("foo").unwrap(), &1);
    assert_eq!(enum_.get_field::<bool>("bar").unwrap(), &false);

    *enum_.get_field_mut("foo").unwrap() = 42;
    assert_eq!(enum_.get_field::<i32>("foo").unwrap(), &42);

    enum_.patch(&EnumValue::new("DoesntMatter").with_field("bar", true));
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
    let mut enum_ = Foo::Foo { foo: 1, bar: false };

    assert_eq!(enum_.get_field::<i32>("foo").unwrap(), &1);
    assert_eq!(enum_.get_field::<bool>("bar").unwrap(), &false);

    *enum_.get_field_mut("foo").unwrap() = 42;
    assert_eq!(enum_.get_field::<i32>("foo").unwrap(), &42);

    enum_.patch(&EnumValue::new("DoesntMatter").with_field("bar", true));
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
