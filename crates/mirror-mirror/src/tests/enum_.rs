use crate::{self as mirror_mirror, enum_::VariantKind, Enum};

use mirror_mirror::{EnumValue, FromReflect, GetField, Reflect};

#[test]
fn enum_value() {
    let mut enum_ = EnumValue::new_struct_variant("Foo")
        .with_field("foo", 1_i32)
        .with_field("bar", false);

    assert_eq!(enum_.get_field::<i32>("foo").unwrap(), &1);
    assert_eq!(enum_.get_field::<bool>("bar").unwrap(), &false);

    *enum_.get_field_mut("foo").unwrap() = 42;
    assert_eq!(enum_.get_field::<i32>("foo").unwrap(), &42);

    enum_.patch(&EnumValue::new_struct_variant("Foo").with_field("bar", true));
    assert_eq!(enum_.get_field::<bool>("bar").unwrap(), &true);

    let enum_ = EnumValue::from_reflect(&enum_).unwrap();
    assert_eq!(enum_.get_field::<i32>("foo").unwrap(), &42);
    assert_eq!(enum_.get_field::<bool>("bar").unwrap(), &true);

    let value = enum_.to_value();
    assert_eq!(value.get_field::<i32>("foo").unwrap(), &42);
    assert_eq!(value.get_field::<bool>("bar").unwrap(), &true);
    let mut has_fields = false;
    for field in value.as_enum().unwrap().fields() {
        has_fields = true;
        match field {
            mirror_mirror::enum_::VariantField::Struct(key, value) => {
                if key == "foo" {
                    assert_eq!(value.downcast_ref::<i32>().unwrap(), &42);
                } else if key == "bar" {
                    assert_eq!(value.downcast_ref::<bool>().unwrap(), &true);
                } else {
                    panic!("unknown field: {key}");
                }
            }
            mirror_mirror::enum_::VariantField::Tuple(_) => panic!("bad variant"),
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

    assert_eq!(enum_.variant_kind(), VariantKind::Struct);

    assert_eq!(enum_.get_field::<i32>("foo").unwrap(), &1);
    assert_eq!(enum_.get_field::<bool>("bar").unwrap(), &false);

    *enum_.get_field_mut("foo").unwrap() = 42;
    assert_eq!(enum_.get_field::<i32>("foo").unwrap(), &42);

    enum_.patch(&EnumValue::new_struct_variant("Foo").with_field("bar", true));
    assert_eq!(enum_.get_field::<bool>("bar").unwrap(), &true);

    // variants with other names are ignored
    enum_.patch(&EnumValue::new_struct_variant("Ignored").with_field("bar", false));
    assert_eq!(enum_.get_field::<bool>("bar").unwrap(), &true);

    assert!(matches!(enum_, Foo::Foo { foo: 42, bar: true }));

    let enum_ = Foo::from_reflect(&enum_).unwrap();
    assert!(matches!(enum_, Foo::Foo { foo: 42, bar: true }));

    let value = enum_.to_value();
    assert_eq!(value.get_field::<i32>("foo").unwrap(), &42);
    assert_eq!(value.get_field::<bool>("bar").unwrap(), &true);
    let mut has_fields = false;
    for field in value.as_enum().unwrap().fields() {
        has_fields = true;
        match field {
            mirror_mirror::enum_::VariantField::Struct(key, value) => {
                if key == "foo" {
                    assert_eq!(value.downcast_ref::<i32>().unwrap(), &42);
                } else if key == "bar" {
                    assert_eq!(value.downcast_ref::<bool>().unwrap(), &true);
                } else {
                    panic!("unknown field: {key}");
                }
            }
            mirror_mirror::enum_::VariantField::Tuple(_) => panic!("bad variant"),
        }
    }
    assert!(has_fields);

    // the variant name must match, even if it has the right fields
    assert!(Foo::from_reflect(
        &EnumValue::new_struct_variant("NotInFoo").with_field("baz", "foo".to_owned()),
    )
    .is_none());
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
    foo.patch(&EnumValue::new_struct_variant("B").with_field("b", false));
    assert!(matches!(dbg!(foo), Foo::B { b: false }));

    // part: static.patch(value)
    let mut foo = Foo::A { a: 1 };
    foo.patch(&EnumValue::new_struct_variant("A").with_field("a", 42));
    assert!(matches!(dbg!(foo), Foo::A { a: 42 }));

    // whole: value.patch(static)
    let mut foo = EnumValue::new_struct_variant("A").with_field("a", 1);
    foo.patch(&Foo::B { b: false });
    assert_eq!(foo.get_field::<bool>("b").unwrap(), &false);

    // part: value.patch(static)
    let mut foo = EnumValue::new_struct_variant("A").with_field("a", 1);
    foo.patch(&Foo::A { a: 42 });
    assert_eq!(foo.get_field::<i32>("a").unwrap(), &42);

    // whole: value.patch(value)
    let mut foo = EnumValue::new_struct_variant("A").with_field("a", 1);
    foo.patch(&EnumValue::new_struct_variant("B").with_field("b", false));
    assert_eq!(foo.variant_name(), "B");
    assert!(foo.get_field::<i32>("a").is_none());
    assert_eq!(foo.get_field::<bool>("b").unwrap(), &false);

    // part: value.patch(value)
    let mut foo = EnumValue::new_struct_variant("A").with_field("a", 1);
    foo.patch(&EnumValue::new_struct_variant("A").with_field("a", 42));
    assert_eq!(foo.get_field::<i32>("a").unwrap(), &42);
}

#[test]
fn static_tuple_enum() {
    #[derive(Reflect, Clone, Debug, PartialEq, Eq)]
    enum Foo {
        A(i32, bool),
        B(String),
    }

    let mut foo = Foo::A(1, true);

    assert_eq!(foo.get_field::<i32>(0).unwrap(), &1);
    assert_eq!(foo.get_field::<bool>(1).unwrap(), &true);
    assert!(foo.element(2).is_none());

    assert_eq!(foo.get_field_mut::<i32>(0).unwrap(), &1);
    assert_eq!(foo.get_field_mut::<bool>(1).unwrap(), &true);
    foo.get_field_mut::<bool>(1).unwrap().patch(&false);
    assert_eq!(foo.get_field_mut::<bool>(1).unwrap(), &false);
    assert!(foo.element_mut(2).is_none());

    // whole: static.patch(static)
    let mut foo = Foo::A(1, true);
    foo.patch(&Foo::B("foo".to_owned()));
    assert!(matches!(dbg!(foo), Foo::B(s) if s == "foo"));

    // part: static.patch(static)
    let mut foo = Foo::A(1, true);
    foo.patch(&Foo::A(42, true));
    assert!(matches!(dbg!(foo), Foo::A(42, true)));

    // whole: static.patch(value)
    let mut foo = Foo::A(1, true);
    foo.patch(&EnumValue::new_tuple_variant("B").with_element("foo"));
    assert!(matches!(dbg!(foo), Foo::B(s) if s == "foo"));

    // part: static.patch(value)
    let mut foo = Foo::A(1, true);
    foo.patch(&EnumValue::new_tuple_variant("A").with_element(42));
    assert!(matches!(dbg!(foo), Foo::A(42, true)));

    // whole: value.patch(static)
    let mut foo = EnumValue::new_tuple_variant("A")
        .with_element(1)
        .with_element(true);
    foo.patch(&Foo::B("foo".to_owned()));
    assert_eq!(foo.get_field::<String>(0).unwrap(), &"foo");
    assert!(foo.element(1).is_none());

    // part: value.patch(static)
    let mut foo = EnumValue::new_tuple_variant("A")
        .with_element(1_i32)
        .with_element(true)
        .with_element("foo");
    foo.patch(&Foo::A(42, true));
    dbg!(&foo);
    assert_eq!(foo.get_field::<i32>(0).unwrap(), &42);
    assert_eq!(foo.get_field::<bool>(1).unwrap(), &true);
    assert_eq!(foo.get_field::<String>(2).unwrap(), &"foo");
    assert!(foo.element(3).is_none());
}
