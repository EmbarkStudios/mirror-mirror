use crate::{FromReflect, Reflect, Struct, Value, ValueInner};
use serde::{Deserialize, Serialize};
use speedy::{Readable, Writable};
use std::{any::Any, fmt};

pub trait Enum: Reflect {
    // TODO(david): change this to have `field` and `field_mut` methods, similar
    // to `Struct`. Should simplify things a lot
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
        Value(ValueInner::EnumValue(Box::new(self.clone())))
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
