use std::{any::Any, collections::HashMap, fmt};

use serde::{Deserialize, Serialize};
use speedy::{Readable, Writable};

#[cfg(test)]
mod tests;

pub trait Reflect: Any + Send + 'static {
    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn as_reflect(&self) -> &dyn Reflect;

    fn as_reflect_mut(&mut self) -> &mut dyn Reflect;

    fn patch(&mut self, value: &dyn Reflect);

    fn as_struct(&self) -> Option<&dyn Struct> {
        None
    }

    fn as_struct_mut(&mut self) -> Option<&mut dyn Struct> {
        None
    }
}

impl dyn Reflect {
    pub fn downcast_ref<T>(&self) -> Option<&T>
    where
        T: Reflect,
    {
        self.as_any().downcast_ref::<T>()
    }

    pub fn downcast_mut<T>(&mut self) -> Option<&mut T>
    where
        T: Reflect,
    {
        self.as_any_mut().downcast_mut::<T>()
    }
}

macro_rules! impl_for_core_types {
    ($($ty:ident)*) => {
        $(
            impl Reflect for $ty {
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
                    if let Some(value) = value.as_any().downcast_ref::<Self>() {
                        *self = value.to_owned();
                    }
                }
            }

            impl FromReflect for $ty {
                fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
                    Some(reflect.downcast_ref::<$ty>()?.to_owned())
                }
            }

            impl IntoValue for $ty {
                fn into_value(self) -> Value {
                    Value(ValueInner::$ty(self))
                }
            }

            impl private::Sealed for $ty {}
        )*
    };
}

impl_for_core_types! {
    usize u8 u16 u32 u64 u128
    i8 i16 i32 i64 i128
    f32 f64
    bool char String
}

pub trait FromReflect: Reflect + Sized {
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self>;
}

pub trait Struct: Reflect {
    fn field(&self, name: &str) -> Option<&dyn Reflect>;

    fn field_mut(&mut self, name: &str) -> Option<&mut dyn Reflect>;

    fn into_value(self) -> StructValue;

    fn fields(&self) -> FieldsIter<'_>;

    fn fields_mut(&mut self) -> FieldsIterMut<'_>;
}

pub struct FieldsIter<'a> {
    iter: Box<dyn Iterator<Item = (&'a str, &'a dyn Reflect)> + 'a>,
}

impl<'a> FieldsIter<'a> {
    pub fn new<I>(iter: I) -> Self
    where
        I: Iterator<Item = (&'a str, &'a dyn Reflect)> + 'a,
    {
        Self {
            iter: Box::new(iter),
        }
    }
}

impl<'a> Iterator for FieldsIter<'a> {
    type Item = (&'a str, &'a dyn Reflect);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

pub struct FieldsIterMut<'a> {
    iter: Box<dyn Iterator<Item = (&'a str, &'a mut dyn Reflect)> + 'a>,
}

impl<'a> FieldsIterMut<'a> {
    pub fn new<I>(iter: I) -> Self
    where
        I: Iterator<Item = (&'a str, &'a mut dyn Reflect)> + 'a,
    {
        Self {
            iter: Box::new(iter),
        }
    }
}

impl<'a> Iterator for FieldsIterMut<'a> {
    type Item = (&'a str, &'a mut dyn Reflect);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

#[derive(Readable, Writable, Serialize, Deserialize, Debug, Clone)]
pub struct StructValue {
    fields: HashMap<String, Value>,
}

impl StructValue {
    pub fn builder() -> StructValueBuilder {
        StructValueBuilder::default()
    }
}

impl Reflect for StructValue {
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
        if let Some(struct_) = value.as_struct() {
            for (name, value) in &mut self.fields {
                if let Some(new_value) = struct_.field(name) {
                    value.patch(new_value);
                }
            }
        }
    }

    fn as_struct(&self) -> Option<&dyn Struct> {
        Some(self)
    }

    fn as_struct_mut(&mut self) -> Option<&mut dyn Struct> {
        Some(self)
    }
}

impl Struct for StructValue {
    fn field(&self, name: &str) -> Option<&dyn Reflect> {
        Some(self.fields.get(name)?)
    }

    fn field_mut(&mut self, name: &str) -> Option<&mut dyn Reflect> {
        Some(self.fields.get_mut(name)?)
    }

    fn into_value(self) -> StructValue {
        self
    }

    fn fields(&self) -> FieldsIter<'_> {
        let iter = self
            .fields
            .iter()
            .map(|(key, value)| (&**key, value.as_reflect()));
        FieldsIter::new(iter)
    }

    fn fields_mut(&mut self) -> FieldsIterMut<'_> {
        let iter = self
            .fields
            .iter_mut()
            .map(|(key, value)| (&**key, value.as_reflect_mut()));
        FieldsIterMut::new(iter)
    }
}

#[derive(Default)]
pub struct StructValueBuilder {
    fields: HashMap<String, Value>,
}

impl StructValueBuilder {
    pub fn set(mut self, name: impl Into<String>, value: impl IntoValue) -> Self {
        self.fields.insert(name.into(), value.into_value());
        self
    }

    pub fn build(self) -> StructValue {
        StructValue {
            fields: self.fields,
        }
    }
}

#[derive(Readable, Writable, Serialize, Deserialize, Clone)]
pub struct Value(ValueInner);

impl Reflect for Value {
    fn as_any(&self) -> &dyn Any {
        self.0.as_any()
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self.0.as_any_mut()
    }

    fn as_reflect(&self) -> &dyn Reflect {
        self.0.as_reflect()
    }

    fn as_reflect_mut(&mut self) -> &mut dyn Reflect {
        self.0.as_reflect_mut()
    }

    #[allow(warnings)]
    fn patch(&mut self, value: &dyn Reflect) {
        self.0.patch(value)
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

macro_rules! value_inner {
    (
        $(#[$m:meta])*
        enum ValueInner {
            $($ident:ident,)*
        }
    ) => {
        $(#[$m])*
        enum ValueInner {
            $($ident($ident),)*
        }

        impl Reflect for ValueInner {
            fn as_any(&self) -> &dyn Any {
                match self {
                    $(
                        Self::$ident(inner) => inner,
                    )*
                }
            }

            fn as_any_mut(&mut self) -> &mut dyn Any {
                match self {
                    $(
                        Self::$ident(inner) => inner,
                    )*
                }
            }

            fn as_reflect(&self) -> &dyn Reflect {
                self
            }

            fn as_reflect_mut(&mut self) -> &mut dyn Reflect {
                self
            }

            fn patch(&mut self, value: &dyn Reflect) {
                match self {
                    $(
                        Self::$ident(inner) => {
                            if let Some(value) = value.downcast_ref::<$ident>() {
                                *inner = value.to_owned();
                            }
                        },
                    )*
                }
            }
        }
    };
}

value_inner! {
    #[allow(non_camel_case_types)]
    #[derive(Readable, Writable, Serialize, Deserialize, Debug, Clone)]
    enum ValueInner {
        usize,
        u8,
        u16,
        u32,
        u64,
        u128,
        i8,
        i16,
        i32,
        i64,
        i128,
        bool,
        char,
        f32,
        f64,
        String,
        StructValue,
    }
}

mod private {
    pub trait Sealed {}
}

pub trait IntoValue: private::Sealed {
    fn into_value(self) -> Value;
}

impl IntoValue for StructValue {
    fn into_value(self) -> Value {
        Value(ValueInner::StructValue(self))
    }
}

impl private::Sealed for StructValue {}
