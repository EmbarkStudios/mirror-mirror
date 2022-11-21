use std::{any::Any, fmt};

use crate::{
    enum_::{EnumFieldsIter, EnumFieldsIterMut},
    Enum, EnumValue, FromReflect, GetField, Reflect, Struct, Value,
};

#[derive(Clone, Debug, PartialEq, Eq)]
enum Foo {
    Foo { foo: i32, bar: bool },
    Bar { baz: String },
}

impl Reflect for Foo {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn as_reflect(&self) -> &dyn Reflect {
        self
    }

    fn as_reflect_mut(&mut self) -> &mut dyn Reflect {
        self
    }

    fn patch(&mut self, value: &dyn Reflect) {
        if let Some(enum_) = value.as_enum() {
            match self {
                Foo::Foo { foo, bar } => {
                    if let Some(new_value) = enum_.field("foo") {
                        foo.patch(new_value);
                    }

                    if let Some(new_value) = enum_.field("bar") {
                        bar.patch(new_value);
                    }
                }
                Foo::Bar { baz } => {
                    if let Some(new_value) = enum_.field("baz") {
                        baz.patch(new_value);
                    }
                }
            }
        }
    }

    fn to_value(&self) -> Value {
        match self {
            Foo::Foo { foo, bar } => EnumValue::new("Foo")
                .with_field("foo", foo.to_owned())
                .with_field("bar", bar.to_owned())
                .to_value(),
            Foo::Bar { baz } => EnumValue::new("Bar")
                .with_field("baz", baz.to_owned())
                .to_value(),
        }
    }

    fn clone_reflect(&self) -> Box<dyn Reflect> {
        Box::new(self.clone())
    }

    fn as_struct(&self) -> Option<&dyn Struct> {
        None
    }

    fn as_struct_mut(&mut self) -> Option<&mut dyn Struct> {
        None
    }

    fn as_enum(&self) -> Option<&dyn Enum> {
        Some(self)
    }

    fn as_enum_mut(&mut self) -> Option<&mut dyn Enum> {
        Some(self)
    }

    fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "{:#?}", self)
        } else {
            write!(f, "{:?}", self)
        }
    }
}

impl Enum for Foo {
    fn variant_name(&self) -> &str {
        match self {
            Foo::Foo { .. } => "Foo",
            Foo::Bar { .. } => "Bar",
        }
    }

    fn field(&self, name: &str) -> Option<&dyn Reflect> {
        match self {
            Foo::Foo { foo, bar } => {
                if name == "foo" {
                    return Some(foo);
                }

                if name == "bar" {
                    return Some(bar);
                }
            }
            Foo::Bar { baz } => {
                if name == "baz" {
                    return Some(baz);
                }
            }
        }

        None
    }

    fn field_mut(&mut self, name: &str) -> Option<&mut dyn Reflect> {
        match self {
            Foo::Foo { foo, bar } => {
                if name == "foo" {
                    return Some(foo);
                }

                if name == "bar" {
                    return Some(bar);
                }
            }
            Foo::Bar { baz } => {
                if name == "baz" {
                    return Some(baz);
                }
            }
        }

        None
    }

    fn fields(&self) -> EnumFieldsIter<'_> {
        match self {
            Foo::Foo { foo, bar } => {
                let iter = [("foo", foo.as_reflect()), ("bar", bar.as_reflect())];
                EnumFieldsIter::new(iter)
            }
            Foo::Bar { baz } => {
                let iter = [("baz", baz.as_reflect())];
                EnumFieldsIter::new(iter)
            }
        }
    }

    fn fields_mut(&mut self) -> EnumFieldsIterMut<'_> {
        match self {
            Foo::Foo { foo, bar } => {
                let iter = [("foo", foo.as_reflect_mut()), ("bar", bar.as_reflect_mut())];
                EnumFieldsIterMut::new(iter)
            }
            Foo::Bar { baz } => {
                let iter = [("baz", baz.as_reflect_mut())];
                EnumFieldsIterMut::new(iter)
            }
        }
    }
}

impl FromReflect for Foo {
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
        let enum_ = reflect.as_enum()?;
        match enum_.variant_name() {
            "Foo" => Some(Self::Foo {
                foo: enum_.get_field::<i32>("foo")?.to_owned(),
                bar: enum_.get_field::<bool>("bar")?.to_owned(),
            }),
            "Bar" => Some(Self::Bar {
                baz: enum_.get_field::<String>("baz")?.to_owned(),
            }),
            _ => None,
        }
    }
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
