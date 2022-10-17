use crate::{FromReflect, Reflect, Struct, Value, ValueInner};
use serde::{Deserialize, Serialize};
use speedy::{Readable, Writable};
use std::{any::Any, fmt};

pub trait Enum: Reflect {
    fn variant(&self) -> Variant<'_>;
    fn variant_mut(&mut self) -> VariantMut<'_>;
}

pub struct Variant<'a> {
    name: &'a str,
    value: &'a dyn Reflect,
    get_field_on_variant: for<'b> fn(&'a dyn Reflect, &'b str) -> Option<&'a dyn Reflect>,
    get_fields_iter: fn(&'a dyn Reflect) -> VariantFieldsIter<'a>,
}

impl<'a> fmt::Debug for Variant<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Variant")
            .field("name", &self.name)
            .field("value", &self.value)
            .finish()
    }
}

impl<'a> Variant<'a> {
    pub fn new(
        name: &'a str,
        value: &'a dyn Reflect,
        get_field_on_variant: for<'b> fn(&'a dyn Reflect, &'b str) -> Option<&'a dyn Reflect>,
        get_fields_iter: fn(&'a dyn Reflect) -> VariantFieldsIter<'a>,
    ) -> Self {
        Self {
            name,
            value,
            get_field_on_variant,
            get_fields_iter,
        }
    }

    pub fn name(&self) -> &str {
        self.name
    }

    pub fn field(&self, name: &str) -> Option<&'a dyn Reflect> {
        (self.get_field_on_variant)(self.value, name)
    }

    pub fn fields(self) -> VariantFieldsIter<'a> {
        (self.get_fields_iter)(self.value)
    }
}

pub struct VariantMut<'a> {
    name: &'a str,
    value: &'a mut dyn Reflect,
    get_field_on_variant: for<'b> fn(&'a mut dyn Reflect, &'b str) -> Option<&'a mut dyn Reflect>,
    get_fields_iter: fn(&'a mut dyn Reflect) -> VariantFieldsIterMut<'a>,
}

impl<'a> VariantMut<'a> {
    pub fn new(
        name: &'a str,
        value: &'a mut dyn Reflect,
        get_field_on_variant: for<'b> fn(
            &'a mut dyn Reflect,
            &'b str,
        ) -> Option<&'a mut dyn Reflect>,
        get_fields_iter: fn(&'a mut dyn Reflect) -> VariantFieldsIterMut<'a>,
    ) -> Self {
        Self {
            name,
            value,
            get_field_on_variant,
            get_fields_iter,
        }
    }

    pub fn name(&self) -> &str {
        self.name
    }

    pub fn field_mut(&'a mut self, name: &str) -> Option<&'a mut dyn Reflect> {
        (self.get_field_on_variant)(self.value, name)
    }

    pub fn into_field_mut(self, name: &str) -> Option<&'a mut dyn Reflect> {
        (self.get_field_on_variant)(self.value, name)
    }

    pub fn fields_mut(&'a mut self) -> VariantFieldsIterMut<'a> {
        (self.get_fields_iter)(self.value)
    }

    pub fn into_fields_mut(self) -> VariantFieldsIterMut<'a> {
        (self.get_fields_iter)(self.value)
    }
}

pub struct VariantFieldsIter<'a> {
    iter: Box<dyn Iterator<Item = (&'a str, &'a dyn Reflect)> + 'a>,
}

impl<'a> VariantFieldsIter<'a> {
    pub fn new<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (&'a str, &'a dyn Reflect)> + 'a,
    {
        Self {
            iter: Box::new(iter.into_iter()),
        }
    }
}

impl<'a> Iterator for VariantFieldsIter<'a> {
    type Item = (&'a str, &'a dyn Reflect);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

pub struct VariantFieldsIterMut<'a> {
    iter: Box<dyn Iterator<Item = (&'a str, &'a mut dyn Reflect)> + 'a>,
}

impl<'a> VariantFieldsIterMut<'a> {
    pub fn new<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (&'a str, &'a mut dyn Reflect)> + 'a,
    {
        Self {
            iter: Box::new(iter.into_iter()),
        }
    }
}

impl<'a> Iterator for VariantFieldsIterMut<'a> {
    type Item = (&'a str, &'a mut dyn Reflect);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Writable, Readable)]
pub struct EnumValue {
    name: String,
    value: Box<Value>,
}

impl EnumValue {
    pub fn new(name: impl Into<String>, value: impl Into<Value>) -> Self {
        Self {
            name: name.into(),
            value: Box::new(value.into()),
        }
    }
}

impl Reflect for EnumValue {
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
            if self.variant().name() == enum_.variant().name() {
                for (key, value) in self.variant_mut().into_fields_mut() {
                    if let Some(new_value) = enum_.variant().field(key) {
                        value.patch(new_value);
                    }
                }
            } else if let Some(value) = EnumValue::from_reflect(value) {
                *self = value;
            }
        }
    }

    fn to_value(&self) -> Value {
        Value(ValueInner::EnumValue(self.clone()))
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

impl Enum for EnumValue {
    fn variant(&self) -> Variant<'_> {
        Variant::new(
            &self.name,
            self.value.as_reflect(),
            Self::get_field_on_variant,
            Self::get_fields_iter,
        )
    }

    fn variant_mut(&mut self) -> VariantMut<'_> {
        VariantMut::new(
            &self.name,
            self.value.as_reflect_mut(),
            Self::get_field_on_variant_mut,
            Self::get_fields_iter_mut,
        )
    }
}

impl EnumValue {
    fn get_field_on_variant<'a>(value: &'a dyn Reflect, name: &str) -> Option<&'a dyn Reflect> {
        if let Some(struct_) = value.as_struct() {
            struct_.field(name)
        } else if let Some(enum_) = value.as_enum() {
            enum_.variant().field(name)
        } else {
            None
        }
    }

    fn get_field_on_variant_mut<'a>(
        value: &'a mut dyn Reflect,
        name: &str,
    ) -> Option<&'a mut dyn Reflect> {
        value.as_struct_mut()?.field_mut(name)
    }

    fn get_fields_iter(value: &dyn Reflect) -> VariantFieldsIter<'_> {
        if let Some(struct_) = value.as_struct() {
            VariantFieldsIter::new(struct_.fields())
        } else if let Some(enum_) = value.as_enum() {
            enum_.variant().fields()
        } else {
            VariantFieldsIter::new(std::iter::empty())
        }
    }

    fn get_fields_iter_mut(value: &mut dyn Reflect) -> VariantFieldsIterMut<'_> {
        if value.as_struct_mut().is_some() {
            VariantFieldsIterMut::new(value.as_struct_mut().unwrap().fields_mut())
        } else if value.as_enum_mut().is_some() {
            value.as_enum_mut().unwrap().variant_mut().into_fields_mut()
        } else {
            VariantFieldsIterMut::new(std::iter::empty())
        }
    }
}

impl FromReflect for EnumValue {
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
        let variant = reflect.as_enum()?.variant();
        Some(Self {
            name: variant.name().to_owned(),
            value: Box::new(variant.value.to_value()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FromReflect, GetField, Struct, StructValue, Value};
    use std::any::Any;

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
}
