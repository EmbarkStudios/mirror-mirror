use crate::enum_::EnumValue;
use crate::enum_::VariantKind;
use crate::get_field::GetField;
use crate::get_field::GetFieldMut;
use crate::DescribeType;
use crate::Enum;
use crate::FromReflect;
use crate::Reflect;

#[test]
fn enum_value() {
    let mut enum_ = EnumValue::new_struct_variant("Foo")
        .with_struct_field("foo", 1_i32)
        .with_struct_field("bar", false)
        .finish();

    assert_eq!(enum_.get_field::<i32>("foo").unwrap(), &1);
    assert_eq!(enum_.get_field::<bool>("bar").unwrap(), &false);

    *enum_.get_field_mut("foo").unwrap() = 42;
    assert_eq!(enum_.get_field::<i32>("foo").unwrap(), &42);

    enum_.patch(
        &EnumValue::new_struct_variant("Foo")
            .with_struct_field("bar", true)
            .finish(),
    );
    assert_eq!(enum_.get_field::<bool>("bar").unwrap(), &true);

    let enum_ = EnumValue::from_reflect(&enum_).unwrap();
    assert_eq!(enum_.get_field::<i32>("foo").unwrap(), &42);
    assert_eq!(enum_.get_field::<bool>("bar").unwrap(), &true);

    let value = enum_.to_value();
    assert_eq!(value.get_field::<i32>("foo").unwrap(), &42);
    assert_eq!(value.get_field::<bool>("bar").unwrap(), &true);
    let mut has_fields = false;
    for field in value.reflect_ref().as_enum().unwrap().fields() {
        has_fields = true;
        match field {
            crate::enum_::VariantField::Struct(key, value) => {
                if key == "foo" {
                    assert_eq!(value.downcast_ref::<i32>().unwrap(), &42);
                } else if key == "bar" {
                    assert_eq!(value.downcast_ref::<bool>().unwrap(), &true);
                } else {
                    panic!("unknown field: {key}");
                }
            }
            crate::enum_::VariantField::Tuple(_) => panic!("bad variant"),
        }
    }
    assert!(has_fields);
}

#[test]
fn static_enum() {
    #[derive(Reflect, Clone, Debug, PartialEq, Eq)]
    #[reflect(crate_name(crate))]
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

    enum_.patch(
        &EnumValue::new_struct_variant("Foo")
            .with_struct_field("bar", true)
            .finish(),
    );
    assert_eq!(enum_.get_field::<bool>("bar").unwrap(), &true);

    // variants with other names are ignored
    enum_.patch(
        &EnumValue::new_struct_variant("Ignored")
            .with_struct_field("bar", false)
            .finish(),
    );
    assert_eq!(enum_.get_field::<bool>("bar").unwrap(), &true);

    assert!(matches!(enum_, Foo::Foo { foo: 42, bar: true }));

    let enum_ = Foo::from_reflect(&enum_).unwrap();
    assert!(matches!(enum_, Foo::Foo { foo: 42, bar: true }));

    let value = enum_.to_value();
    assert_eq!(value.get_field::<i32>("foo").unwrap(), &42);
    assert_eq!(value.get_field::<bool>("bar").unwrap(), &true);
    let mut has_fields = false;
    for field in value.reflect_ref().as_enum().unwrap().fields() {
        has_fields = true;
        match field {
            crate::enum_::VariantField::Struct(key, value) => {
                if key == "foo" {
                    assert_eq!(value.downcast_ref::<i32>().unwrap(), &42);
                } else if key == "bar" {
                    assert_eq!(value.downcast_ref::<bool>().unwrap(), &true);
                } else {
                    panic!("unknown field: {key}");
                }
            }
            crate::enum_::VariantField::Tuple(_) => panic!("bad variant"),
        }
    }
    assert!(has_fields);

    // the variant name must match, even if it has the right fields
    assert!(Foo::from_reflect(
        &EnumValue::new_struct_variant("NotInFoo")
            .with_struct_field("baz", "foo".to_owned())
            .finish(),
    )
    .is_none());
}

#[test]
fn patching() {
    #[derive(Reflect, Clone, Debug, PartialEq, Eq)]
    #[reflect(crate_name(crate))]
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
    foo.patch(
        &EnumValue::new_struct_variant("B")
            .with_struct_field("b", false)
            .finish(),
    );
    assert!(matches!(dbg!(foo), Foo::B { b: false }));

    // part: static.patch(value)
    let mut foo = Foo::A { a: 1 };
    foo.patch(
        &EnumValue::new_struct_variant("A")
            .with_struct_field("a", 42)
            .finish(),
    );
    assert!(matches!(dbg!(foo), Foo::A { a: 42 }));

    // whole: value.patch(static)
    let mut foo = EnumValue::new_struct_variant("A")
        .with_struct_field("a", 1)
        .finish();
    foo.patch(&Foo::B { b: false });
    assert_eq!(foo.get_field::<bool>("b").unwrap(), &false);

    // part: value.patch(static)
    let mut foo = EnumValue::new_struct_variant("A")
        .with_struct_field("a", 1)
        .finish();
    foo.patch(&Foo::A { a: 42 });
    assert_eq!(foo.get_field::<i32>("a").unwrap(), &42);

    // whole: value.patch(value)
    let mut foo = EnumValue::new_struct_variant("A")
        .with_struct_field("a", 1)
        .finish();
    foo.patch(
        &EnumValue::new_struct_variant("B")
            .with_struct_field("b", false)
            .finish(),
    );
    assert_eq!(foo.variant_name(), "B");
    assert!(foo.get_field::<i32>("a").is_none());
    assert_eq!(foo.get_field::<bool>("b").unwrap(), &false);

    // part: value.patch(value)
    let mut foo = EnumValue::new_struct_variant("A")
        .with_struct_field("a", 1)
        .finish();
    foo.patch(
        &EnumValue::new_struct_variant("A")
            .with_struct_field("a", 42)
            .finish(),
    );
    assert_eq!(foo.get_field::<i32>("a").unwrap(), &42);
}

#[test]
fn static_tuple_enum() {
    #[derive(Reflect, Clone, Debug, PartialEq, Eq)]
    #[reflect(crate_name(crate))]
    enum Foo {
        A(i32, bool),
        B(String),
    }

    let mut foo = Foo::A(1, true);

    assert_eq!(foo.get_field::<i32>(0).unwrap(), &1);
    assert_eq!(foo.get_field::<bool>(1).unwrap(), &true);
    assert!(foo.field_at(2).is_none());

    assert_eq!(foo.get_field_mut::<i32>(0).unwrap(), &1);
    assert_eq!(foo.get_field_mut::<bool>(1).unwrap(), &true);
    foo.get_field_mut::<bool>(1).unwrap().patch(&false);
    assert_eq!(foo.get_field_mut::<bool>(1).unwrap(), &false);
    assert!(foo.field_at_mut(2).is_none());

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
    foo.patch(
        &EnumValue::new_tuple_variant("B")
            .with_tuple_field("foo")
            .finish(),
    );
    assert!(matches!(dbg!(foo), Foo::B(s) if s == "foo"));

    // part: static.patch(value)
    let mut foo = Foo::A(1, true);
    foo.patch(
        &EnumValue::new_tuple_variant("A")
            .with_tuple_field(42)
            .finish(),
    );
    assert!(matches!(dbg!(foo), Foo::A(42, true)));

    // whole: value.patch(static)
    let mut foo = EnumValue::new_tuple_variant("A")
        .with_tuple_field(1)
        .with_tuple_field(true)
        .finish();
    foo.patch(&Foo::B("foo".to_owned()));
    assert_eq!(foo.get_field::<String>(0).unwrap(), &"foo");
    assert!(foo.field_at(1).is_none());

    // part: value.patch(static)
    let mut foo = EnumValue::new_tuple_variant("A")
        .with_tuple_field(1_i32)
        .with_tuple_field(true)
        .with_tuple_field("foo")
        .finish();
    foo.patch(&Foo::A(42, true));
    assert_eq!(foo.get_field::<i32>(0).unwrap(), &42);
    assert_eq!(foo.get_field::<bool>(1).unwrap(), &true);
    assert_eq!(foo.get_field::<String>(2).unwrap(), &"foo");
    assert!(foo.field_at(3).is_none());
}

#[test]
fn unit_variant() {
    #[derive(Reflect, Clone, Debug, PartialEq, Eq)]
    #[reflect(crate_name(crate))]
    enum Foo {
        A,
        B,
    }

    // static.patch(static)
    let mut foo = Foo::A;
    foo.patch(&Foo::B);
    assert!(matches!(dbg!(foo), Foo::B));

    // static.patch(value)
    let mut foo = Foo::A;
    foo.patch(&EnumValue::new_unit_variant("B"));
    assert!(matches!(dbg!(foo), Foo::B));

    // value.patch(static)
    let mut foo = EnumValue::new_unit_variant("A");
    foo.patch(&Foo::B);
    assert_eq!(foo.variant_name(), "B");

    // value.patch(value)
    let mut foo = EnumValue::new_unit_variant("A");
    foo.patch(&EnumValue::new_unit_variant("B"));
    assert_eq!(foo.variant_name(), "B");

    let mut foo = Foo::A;

    foo.patch(&EnumValue::new_unit_variant("Unknown"));
    assert!(matches!(dbg!(&foo), Foo::A));

    let value = foo.to_value();
    let value = value.reflect_ref().as_enum().unwrap();
    assert_eq!(value.variant_kind(), VariantKind::Unit);
    assert_eq!(value.variant_name(), "A");
}

#[test]
fn option() {
    assert_eq!(Some(1).variant_name(), "Some");
    assert_eq!(Some(1).variant_kind(), VariantKind::Tuple);
    assert_eq!(format!("{:?}", Some(1).as_reflect()), "Some(1)");

    assert_eq!(None::<i32>.variant_name(), "None");
    assert_eq!(None::<i32>.variant_kind(), VariantKind::Unit);
    assert_eq!(format!("{:?}", None::<i32>.as_reflect()), "None");

    let some_value = Some(1).to_value();
    let some_value: &dyn Enum = some_value.reflect_ref().as_enum().unwrap();
    assert_eq!(some_value.variant_name(), "Some");
    assert_eq!(some_value.variant_kind(), VariantKind::Tuple);
    assert_eq!(some_value.get_field::<i32>(0).unwrap(), &1);
    assert!(some_value.field_at(1).is_none());

    let none_value = None::<i32>.to_value();
    let none_value = none_value.reflect_ref().as_enum().unwrap();
    assert_eq!(none_value.variant_name(), "None");
    assert_eq!(none_value.variant_kind(), VariantKind::Unit);
    assert!(none_value.field_at(0).is_none());

    assert_eq!(
        Some(1)
            .clone_reflect()
            .reflect_ref()
            .as_enum()
            .unwrap()
            .variant_name(),
        "Some"
    );
    assert_eq!(
        None::<i32>
            .clone_reflect()
            .reflect_ref()
            .as_enum()
            .unwrap()
            .variant_name(),
        "None"
    );

    let mut value = Some(1);
    value.patch(&Some(42));
    assert_eq!(value, Some(42));

    let mut value = Some(1);
    value.patch(&None::<i32>);
    assert_eq!(value, None);

    let mut value = Some(1);
    value.patch(&EnumValue::new_unit_variant("None"));
    assert_eq!(value, None);

    let mut value = Some(1);
    value.patch(&None::<i32>.to_value());
    assert_eq!(value, None);

    let mut value = Some(1);
    value.patch(&EnumValue::new_unit_variant("None").to_value());
    assert_eq!(value, None);

    let mut value = Some(1).to_value();
    value.patch(&None::<i32>.to_value());
    assert_eq!(Option::<i32>::from_reflect(&value).unwrap(), None);

    let mut value = Some(1).to_value();
    value.patch(&EnumValue::new_unit_variant("None").to_value());
    assert_eq!(Option::<i32>::from_reflect(&value).unwrap(), None);

    let mut value = None::<i32>;
    value.patch(&Some(42));
    assert_eq!(value, Some(42));

    let mut value = None::<i32>;
    value.patch(&None::<i32>);
    assert_eq!(value, None);

    assert_eq!(Option::<i32>::from_reflect(&Some(1)).unwrap(), Some(1));
    assert_eq!(Option::<i32>::from_reflect(&None::<i32>).unwrap(), None);

    assert_eq!(Some(1).fields_len(), 1);
    assert_eq!(None::<i32>.fields_len(), 0);
}

#[test]
fn from_reflect_with_value() {
    #[derive(Debug, Clone, Reflect)]
    #[reflect(crate_name(crate))]
    pub enum Foo {
        Struct { number: Number },
        Tuple(Number),
    }

    #[derive(Debug, Clone, Reflect)]
    #[reflect(crate_name(crate))]
    pub enum Number {
        One,
        Two,
        Three,
    }

    let value = EnumValue::new_struct_variant("Struct")
        .with_struct_field("number", Number::One)
        .finish();
    assert!(Foo::from_reflect(&value).is_some());

    let value = EnumValue::new_tuple_variant("Tuple")
        .with_tuple_field(Number::One)
        .finish();
    assert!(Foo::from_reflect(&value).is_some());
}

#[test]
fn default_value_for_enum_variant_type() {
    #[derive(Debug, Clone, Reflect, PartialEq)]
    #[reflect(crate_name(crate))]
    pub enum Foo {
        A,
        B(i32, String),
        C { a: f32, b: Option<bool> },
    }

    let type_ = <Foo as DescribeType>::type_descriptor();
    let enum_type = type_.as_enum().unwrap();

    assert_eq!(
        Foo::from_reflect(
            &enum_type
                .variant("A")
                .expect("no variant A")
                .default_value()
                .expect("can't make default value for opaque type")
        )
        .expect("from_reflect failed"),
        Foo::A,
    );

    assert_eq!(
        Foo::from_reflect(
            &enum_type
                .variant("B")
                .expect("no variant B")
                .default_value()
                .expect("can't make default value for opaque type")
        )
        .expect("from_reflect failed"),
        Foo::B(0, "".to_owned()),
    );

    assert_eq!(
        Foo::from_reflect(
            &enum_type
                .variant("C")
                .expect("no variant C")
                .default_value()
                .expect("can't make default value for opaque type")
        )
        .expect("from_reflect failed"),
        Foo::C { a: 0.0, b: None },
    );
}
