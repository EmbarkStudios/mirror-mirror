use std::{any::Any, collections::HashMap, fmt};

use crate::{Enum, FromReflect, Reflect, Tuple, Value};
use serde::{Deserialize, Serialize};
use speedy::{Readable, Writable};

pub trait Struct: Reflect {
    fn field(&self, name: &str) -> Option<&dyn Reflect>;
    fn field_mut(&mut self, name: &str) -> Option<&mut dyn Reflect>;

    fn fields(&self) -> StructFieldsIter<'_>;
    fn fields_mut(&mut self) -> StructFieldsIterMut<'_>;
}

pub struct StructFieldsIter<'a> {
    iter: Box<dyn Iterator<Item = (&'a str, &'a dyn Reflect)> + 'a>,
}

impl<'a> StructFieldsIter<'a> {
    pub fn new<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (&'a str, &'a dyn Reflect)> + 'a,
    {
        Self {
            iter: Box::new(iter.into_iter()),
        }
    }
}

impl<'a> Iterator for StructFieldsIter<'a> {
    type Item = (&'a str, &'a dyn Reflect);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

pub struct StructFieldsIterMut<'a> {
    iter: Box<dyn Iterator<Item = (&'a str, &'a mut dyn Reflect)> + 'a>,
}

impl<'a> StructFieldsIterMut<'a> {
    pub fn new<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (&'a str, &'a mut dyn Reflect)> + 'a,
    {
        Self {
            iter: Box::new(iter.into_iter()),
        }
    }
}

impl<'a> Iterator for StructFieldsIterMut<'a> {
    type Item = (&'a str, &'a mut dyn Reflect);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

#[derive(Default, Readable, Writable, Serialize, Deserialize, Debug, Clone)]
pub struct StructValue {
    fields: HashMap<String, Value>,
}

impl StructValue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_field(mut self, name: impl Into<String>, value: impl Into<Value>) -> Self {
        self.set_field(name, value);
        self
    }

    pub fn set_field(&mut self, name: impl Into<String>, value: impl Into<Value>) {
        self.fields.insert(name.into(), value.into());
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

    fn as_tuple(&self) -> Option<&dyn Tuple> {
        None
    }

    fn as_tuple_mut(&mut self) -> Option<&mut dyn Tuple> {
        None
    }

    fn as_struct(&self) -> Option<&dyn Struct> {
        Some(self)
    }

    fn as_struct_mut(&mut self) -> Option<&mut dyn Struct> {
        Some(self)
    }

    fn as_enum(&self) -> Option<&dyn Enum> {
        None
    }

    fn as_enum_mut(&mut self) -> Option<&mut dyn Enum> {
        None
    }

    fn patch(&mut self, value: &dyn Reflect) {
        if let Some(struct_) = value.as_struct() {
            for (name, value) in self.fields_mut() {
                if let Some(new_value) = struct_.field(name) {
                    value.patch(new_value);
                }
            }
        }
    }

    fn to_value(&self) -> Value {
        self.clone().into()
    }

    fn clone_reflect(&self) -> Box<dyn Reflect> {
        Box::new(self.clone())
    }

    fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "{:#?}", self)
        } else {
            write!(f, "{:?}", self)
        }
    }
}

impl Struct for StructValue {
    fn field(&self, name: &str) -> Option<&dyn Reflect> {
        Some(self.fields.get(name)?)
    }

    fn field_mut(&mut self, name: &str) -> Option<&mut dyn Reflect> {
        Some(self.fields.get_mut(name)?)
    }

    fn fields(&self) -> StructFieldsIter<'_> {
        let iter = self
            .fields
            .iter()
            .map(|(key, value)| (&**key, value.as_reflect()));
        StructFieldsIter::new(iter)
    }

    fn fields_mut(&mut self) -> StructFieldsIterMut<'_> {
        let iter = self
            .fields
            .iter_mut()
            .map(|(key, value)| (&**key, value.as_reflect_mut()));
        StructFieldsIterMut::new(iter)
    }
}

impl FromReflect for StructValue {
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
        let struct_ = reflect.as_struct()?;
        let this = struct_
            .fields()
            .fold(StructValue::default(), |builder, (name, value)| {
                builder.with_field(name, value.to_value())
            });
        Some(this)
    }
}
