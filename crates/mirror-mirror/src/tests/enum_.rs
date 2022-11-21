use std::{any::Any, fmt};

use crate::{
    enum_::{Variant, VariantFieldsIter, VariantFieldsIterMut, VariantMut},
    Enum, EnumValue, FromReflect, GetField, Reflect, Struct, StructValue, Value,
};

#[derive(Clone, Debug, PartialEq, Eq)]
enum Foo {
    A { a: i32 },
    B { b: bool },
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
            let variant = enum_.variant();
            match self {
                Foo::A { a } => {
                    if variant.name() == "A" {
                        if let Some(new_value) = variant.field("a") {
                            a.patch(new_value);
                        }
                        return;
                    }
                }
                Foo::B { b } => {
                    if variant.name() == "B" {
                        if let Some(new_value) = variant.field("b") {
                            b.patch(new_value);
                        }
                        return;
                    }
                }
            }
            if let Some(new_value) = Self::from_reflect(enum_.as_reflect()) {
                *self = new_value;
            }
        }
    }

    fn to_value(&self) -> Value {
        match self {
            Foo::A { a } => {
                EnumValue::new("A", StructValue::default().with_field("a", a.to_owned())).into()
            }
            Foo::B { b } => {
                EnumValue::new("B", StructValue::default().with_field("b", b.to_owned())).into()
            }
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
    fn variant(&self) -> Variant<'_> {
        match self {
            this @ Foo::A { .. } => {
                fn get_field_on_variant<'a>(
                    this: &'a dyn Reflect,
                    name: &str,
                ) -> Option<&'a dyn Reflect> {
                    match this.downcast_ref().unwrap() {
                        Foo::A { a } => (name == "a").then_some(a),
                        _ => unreachable!(),
                    }
                }

                fn get_fields_iter(this: &dyn Reflect) -> VariantFieldsIter<'_> {
                    match this.downcast_ref().unwrap() {
                        Foo::A { a } => {
                            let iter = [("a", a.as_reflect())];
                            VariantFieldsIter::new(iter)
                        }
                        _ => unreachable!(),
                    }
                }

                Variant::new("A", this, get_field_on_variant, get_fields_iter)
            }
            this @ Foo::B { .. } => {
                fn get_field_on_variant<'a>(
                    this: &'a dyn Reflect,
                    name: &str,
                ) -> Option<&'a dyn Reflect> {
                    match this.downcast_ref().unwrap() {
                        Foo::B { b } => (name == "b").then_some(b),
                        _ => unreachable!(),
                    }
                }

                fn get_fields_iter(this: &dyn Reflect) -> VariantFieldsIter<'_> {
                    match this.downcast_ref().unwrap() {
                        Foo::B { b } => {
                            let iter = [("b", b.as_reflect())];
                            VariantFieldsIter::new(iter)
                        }
                        _ => unreachable!(),
                    }
                }

                Variant::new("B", this, get_field_on_variant, get_fields_iter)
            }
        }
    }

    fn variant_mut(&mut self) -> VariantMut<'_> {
        match self {
            this @ Foo::A { .. } => {
                fn get_field_on_variant<'a>(
                    this: &'a mut dyn Reflect,
                    name: &str,
                ) -> Option<&'a mut dyn Reflect> {
                    match this.downcast_mut().unwrap() {
                        Foo::A { a } => {
                            if name == "a" {
                                return Some(a);
                            }
                            None
                        }
                        _ => unreachable!(),
                    }
                }

                fn get_fields_iter(this: &mut dyn Reflect) -> VariantFieldsIterMut<'_> {
                    match this.downcast_mut().unwrap() {
                        Foo::A { a } => {
                            let iter = [("a", a.as_reflect_mut())];
                            VariantFieldsIterMut::new(iter)
                        }
                        _ => unreachable!(),
                    }
                }

                VariantMut::new("A", this, get_field_on_variant, get_fields_iter)
            }
            this @ Foo::B { .. } => {
                fn get_field_on_variant<'a>(
                    this: &'a mut dyn Reflect,
                    name: &str,
                ) -> Option<&'a mut dyn Reflect> {
                    match this.downcast_mut().unwrap() {
                        Foo::B { b } => (name == "b").then_some(b),
                        _ => unreachable!(),
                    }
                }

                fn get_fields_iter(this: &mut dyn Reflect) -> VariantFieldsIterMut<'_> {
                    match this.downcast_mut().unwrap() {
                        Foo::B { b } => {
                            let iter = [("b", b.as_reflect_mut())];
                            VariantFieldsIterMut::new(iter)
                        }
                        _ => unreachable!(),
                    }
                }

                VariantMut::new("B", this, get_field_on_variant, get_fields_iter)
            }
        }
    }
}

impl FromReflect for Foo {
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
        let enum_ = reflect.as_enum()?;
        let variant = enum_.variant();
        match variant.name() {
            "A" => Some(Foo::A {
                a: variant.field("a")?.downcast_ref::<i32>()?.to_owned(),
            }),
            "B" => Some(Foo::B {
                b: variant.field("b")?.downcast_ref::<bool>()?.to_owned(),
            }),
            _ => None,
        }
    }
}

macro_rules! get_field_on_variant {
    ($expr:expr, $ident:ident, $ty:ty) => {
        *$expr.get_field::<$ty>(stringify!($ident)).unwrap()
    };
}

#[test]
fn accessing_variants() {
    let foo = Foo::A { a: 42 };
    let enum_ = foo.as_enum().unwrap();

    assert_eq!(get_field_on_variant!(enum_, a, i32), 42);
}

#[test]
fn accessing_variants_mut() {
    let mut foo = Foo::A { a: 42 };
    let enum_ = foo.as_enum_mut().unwrap();
    let mut variant = enum_.variant_mut();

    variant.field_mut("a").unwrap().patch(&1337);

    assert!(matches!(dbg!(foo), Foo::A { a: 1337 }));
}

#[test]
fn fields_on_variant() {
    let mut foo = Foo::A { a: 42 };

    for (name, value) in foo.variant().fields() {
        if name == "a" {
            assert_eq!(value.downcast_ref::<i32>().unwrap(), &42);
        } else {
            panic!("unknown field on variant {name:?}");
        }
    }

    for (name, value) in foo.variant_mut().into_fields_mut() {
        if name == "a" {
            assert_eq!(value.downcast_ref::<i32>().unwrap(), &42);
        } else {
            panic!("unknown field on variant {name:?}");
        }
    }
}

#[test]
fn patch() {
    let mut foo = Foo::A { a: 42 };
    foo.patch(&Foo::A { a: 1337 });
    assert!(matches!(dbg!(foo), Foo::A { a: 1337 }));

    let mut foo = Foo::A { a: 1337 };
    foo.patch(&EnumValue::new(
        "A",
        StructValue::default().with_field("a", 42),
    ));
    assert!(matches!(dbg!(foo), Foo::A { a: 42 }));
}

#[test]
fn patch_value() {
    let mut value = EnumValue::new("A", StructValue::default().with_field("a", 42));
    value.patch(&Foo::A { a: 1337 });
    assert_eq!(get_field_on_variant!(value, a, i32), 1337,);

    let mut value = EnumValue::new("A", StructValue::default().with_field("a", 42));
    value.patch(&EnumValue::new(
        "A",
        StructValue::default().with_field("a", 1337),
    ));
    assert_eq!(get_field_on_variant!(value, a, i32), 1337,);
}

#[test]
fn patch_change_variant() {
    let mut foo = Foo::A { a: 42 };

    foo.patch(&Foo::B { b: false });
    assert!(matches!(dbg!(&foo), Foo::B { b: false }));

    foo.patch(&Foo::A { a: 1337 });
    assert!(matches!(dbg!(&foo), Foo::A { a: 1337 }));

    foo.patch(&EnumValue::new(
        "B",
        StructValue::new().with_field("b", true),
    ));
    assert!(matches!(dbg!(&foo), Foo::B { b: true }));

    foo.patch(&EnumValue::new("A", StructValue::new().with_field("a", 42)));
    assert!(matches!(dbg!(&foo), Foo::A { a: 42 }));
}

#[test]
#[allow(clippy::bool_assert_comparison)]
fn patch_value_change_variant_isnt_allowed() {
    let mut foo = EnumValue::new("A", StructValue::new().with_field("a", 42));

    foo.patch(&Foo::B { b: false });
    assert_eq!(false, get_field_on_variant!(&foo, b, bool));

    foo.patch(&Foo::A { a: 1337 });
    assert_eq!(1337, get_field_on_variant!(&foo, a, i32));

    foo.patch(&EnumValue::new(
        "B",
        StructValue::new().with_field("b", true),
    ));
    assert_eq!(true, get_field_on_variant!(&foo, b, bool));

    foo.patch(&EnumValue::new("A", StructValue::new().with_field("a", 42)));
    assert_eq!(42, get_field_on_variant!(&foo, a, i32));
}

#[test]
fn to_value() {
    let foo = Foo::A { a: 42 };
    let value = foo.to_value();
    let new_foo = Foo::from_reflect(&value).unwrap();
    assert_eq!(foo, new_foo);
}

#[test]
fn from_reflect() {
    let value = EnumValue::new("A", StructValue::default().with_field("a", 42));
    let foo = Foo::from_reflect(&value).unwrap();
    assert!(matches!(dbg!(foo), Foo::A { a: 42 }));
}

#[test]
fn value_from_reflect() {
    let foo = Foo::A { a: 42 };
    let value = EnumValue::from_reflect(&foo).unwrap();
    assert_eq!(get_field_on_variant!(value, a, i32), 42);

    let new_foo = Foo::from_reflect(&value).unwrap();
    assert_eq!(foo, new_foo);
}

#[test]
fn value_field_mut() {
    let mut value = EnumValue::new("A", StructValue::default().with_field("a", 42));
    value.variant_mut().field_mut("a").unwrap().patch(&1337);
    assert_eq!(get_field_on_variant!(value, a, i32), 1337);

    let foo = Foo::from_reflect(&value).unwrap();
    assert!(matches!(dbg!(foo), Foo::A { a: 1337 }));
}

#[test]
fn value_fields() {
    let mut value = EnumValue::new("A", StructValue::default().with_field("a", 42));

    for (name, value) in value.variant().fields() {
        if name == "a" {
            assert_eq!(value.downcast_ref::<i32>().unwrap(), &42);
        } else {
            panic!("unknown field on variant {name:?}");
        }
    }

    for (name, value) in value.variant_mut().into_fields_mut() {
        if name == "a" {
            assert_eq!(value.downcast_ref::<i32>().unwrap(), &42);
        } else {
            panic!("unknown field on variant {name:?}");
        }
    }
}

#[test]
fn iterating_all_kinds_of_fields() {
    let mut value = Foo::A { a: 42 };
    assert_eq!(value.variant().fields().count(), 1);
    assert_eq!(value.variant_mut().into_fields_mut().count(), 1);

    let mut value = EnumValue::new("A", StructValue::default().with_field("a", 42));
    assert_eq!(value.variant().fields().count(), 1);
    assert_eq!(value.variant_mut().into_fields_mut().count(), 1);

    let mut value = EnumValue::from_reflect(&(Foo::A { a: 42 })).unwrap();
    assert_eq!(value.variant().fields().count(), 1);
    assert_eq!(value.variant_mut().into_fields_mut().count(), 1);
}

#[test]
fn accessing_unknown_field() {
    let foo = Foo::A { a: 42 };
    assert!(foo.variant().field("b").is_none());

    let value = EnumValue::new("A", StructValue::default().with_field("a", 42));
    assert!(value.variant().field("b").is_none());
}
