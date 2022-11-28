use std::{any::Any, collections::HashMap, fmt};

use crate::{Enum, FromReflect, List, PairIter, PairIterMut, Reflect, Tuple, TupleStruct, Value};
use serde::{Deserialize, Serialize};
use speedy::{Readable, Writable};

pub trait Struct: Reflect {
    fn field(&self, name: &str) -> Option<&dyn Reflect>;
    fn field_mut(&mut self, name: &str) -> Option<&mut dyn Reflect>;

    fn fields(&self) -> PairIter<'_>;
    fn fields_mut(&mut self) -> PairIterMut<'_>;
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

    fn as_tuple_struct(&self) -> Option<&dyn TupleStruct> {
        None
    }

    fn as_tuple_struct_mut(&mut self) -> Option<&mut dyn TupleStruct> {
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

    fn as_list(&self) -> Option<&dyn List> {
        None
    }

    fn as_list_mut(&mut self) -> Option<&mut dyn List> {
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

    fn fields(&self) -> PairIter<'_> {
        let iter = self
            .fields
            .iter()
            .map(|(key, value)| (&**key, value.as_reflect()));
        PairIter::new(iter)
    }

    fn fields_mut(&mut self) -> PairIterMut<'_> {
        let iter = self
            .fields
            .iter_mut()
            .map(|(key, value)| (&**key, value.as_reflect_mut()));
        PairIterMut::new(iter)
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

impl<S, V> FromIterator<(S, V)> for StructValue
where
    S: Into<String>,
    V: Reflect,
{
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (S, V)>,
    {
        let mut out = Self::default();
        for (name, value) in iter {
            out.set_field(name, value.to_value());
        }
        out
    }
}
