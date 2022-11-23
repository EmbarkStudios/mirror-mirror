use crate::{
    FromReflect, PairIter, PairIterMut, Reflect, Struct, StructValue, Tuple, TupleStruct, Value,
};
use serde::{Deserialize, Serialize};
use speedy::{Readable, Writable};
use std::{any::Any, fmt};

pub trait Enum: Reflect {
    fn variant_name(&self) -> &str;

    fn field(&self, name: &str) -> Option<&dyn Reflect>;
    fn field_mut(&mut self, name: &str) -> Option<&mut dyn Reflect>;

    fn fields(&self) -> PairIter<'_>;
    fn fields_mut(&mut self) -> PairIterMut<'_>;
}

#[derive(Clone, Debug, Serialize, Deserialize, Writable, Readable)]
pub struct EnumValue {
    name: String,
    struct_: StructValue,
}

impl EnumValue {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            struct_: Default::default(),
        }
    }

    pub fn with_field(mut self, name: impl Into<String>, value: impl Into<Value>) -> Self {
        self.set_field(name, value);
        self
    }

    pub fn set_field(&mut self, name: impl Into<String>, value: impl Into<Value>) {
        self.struct_.set_field(name, value);
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
            if self.variant_name() == enum_.variant_name() {
                for (name, value) in self.fields_mut() {
                    if let Some(new_value) = enum_.field(name) {
                        value.patch(new_value);
                    }
                }
            } else if let Some(new) = Self::from_reflect(value) {
                *self = new;
            }
        }
    }

    fn to_value(&self) -> Value {
        self.clone().into()
    }

    fn clone_reflect(&self) -> Box<dyn Reflect> {
        Box::new(self.clone())
    }

    fn as_tuple(&self) -> Option<&dyn Tuple> {
        None
    }

    fn as_tuple_mut(&mut self) -> Option<&mut dyn Tuple> {
        None
    }

    fn as_struct(&self) -> Option<&dyn Struct> {
        None
    }

    fn as_struct_mut(&mut self) -> Option<&mut dyn Struct> {
        None
    }

    fn as_tuple_struct(&self) -> Option<&dyn TupleStruct> {
        None
    }

    fn as_tuple_struct_mut(&mut self) -> Option<&mut dyn TupleStruct> {
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
    fn variant_name(&self) -> &str {
        &self.name
    }

    fn field(&self, name: &str) -> Option<&dyn Reflect> {
        self.struct_.field(name)
    }

    fn field_mut(&mut self, name: &str) -> Option<&mut dyn Reflect> {
        self.struct_.field_mut(name)
    }

    fn fields(&self) -> PairIter<'_> {
        PairIter::new(self.struct_.fields())
    }

    fn fields_mut(&mut self) -> PairIterMut<'_> {
        PairIterMut::new(self.struct_.fields_mut())
    }
}

impl FromReflect for EnumValue {
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
        let enum_ = reflect.as_enum()?;
        let struct_ = enum_
            .fields()
            .fold(StructValue::default(), |builder, (name, value)| {
                builder.with_field(name, value.to_value())
            });
        Some(EnumValue {
            name: enum_.variant_name().to_owned(),
            struct_,
        })
    }
}
