use crate::enum_::EnumValue;
use crate::struct_::StructValue;
use crate::struct_::TupleStructValue;
use crate::tuple::TupleValue;
use crate::type_info::graph::Id;
use crate::type_info::graph::TypeInfoGraph;
use crate::type_info::graph::TypeInfoNode;
use crate::FromReflect;
use crate::Reflect;
use crate::ReflectMut;
use crate::ReflectRef;
use crate::ScalarMut;
use crate::ScalarRef;
use crate::TypeInfoRoot;
use crate::Typed;
use ordered_float::OrderedFloat;
use serde::Deserialize;
use serde::Serialize;
use speedy::Readable;
use speedy::Writable;
use std::any::Any;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt;

#[allow(non_camel_case_types)]
#[derive(Readable, Writable, Serialize, Deserialize, Debug, Clone)]
pub enum Value {
    usize(usize),
    u8(u8),
    u16(u16),
    u32(u32),
    u64(u64),
    u128(u128),
    i8(i8),
    i16(i16),
    i32(i32),
    i64(i64),
    i128(i128),
    bool(bool),
    char(char),
    f32(f32),
    f64(f64),
    String(String),
    StructValue(Box<StructValue>),
    EnumValue(Box<EnumValue>),
    TupleStructValue(TupleStructValue),
    TupleValue(TupleValue),
    List(Vec<Value>),
    Map(BTreeMap<Value, Value>),
}

impl FromReflect for Value {
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
        Some(reflect.to_value())
    }
}

#[allow(non_camel_case_types)]
#[derive(Eq, PartialEq, PartialOrd, Ord)]
enum OrdEqValue<'a> {
    usize(usize),
    u8(u8),
    u16(u16),
    u32(u32),
    u64(u64),
    u128(u128),
    i8(i8),
    i16(i16),
    i32(i32),
    i64(i64),
    i128(i128),
    bool(bool),
    char(char),
    f32(OrderedFloat<f32>),
    f64(OrderedFloat<f64>),
    String(&'a str),
    StructValue(&'a StructValue),
    EnumValue(&'a EnumValue),
    TupleStructValue(&'a TupleStructValue),
    TupleValue(&'a TupleValue),
    List(&'a [Value]),
    Map(&'a BTreeMap<Value, Value>),
}

impl<'a> From<&'a Value> for OrdEqValue<'a> {
    fn from(value: &'a Value) -> Self {
        match value {
            Value::usize(inner) => OrdEqValue::usize(*inner),
            Value::u8(inner) => OrdEqValue::u8(*inner),
            Value::u16(inner) => OrdEqValue::u16(*inner),
            Value::u32(inner) => OrdEqValue::u32(*inner),
            Value::u64(inner) => OrdEqValue::u64(*inner),
            Value::u128(inner) => OrdEqValue::u128(*inner),
            Value::i8(inner) => OrdEqValue::i8(*inner),
            Value::i16(inner) => OrdEqValue::i16(*inner),
            Value::i32(inner) => OrdEqValue::i32(*inner),
            Value::i64(inner) => OrdEqValue::i64(*inner),
            Value::i128(inner) => OrdEqValue::i128(*inner),
            Value::bool(inner) => OrdEqValue::bool(*inner),
            Value::char(inner) => OrdEqValue::char(*inner),
            Value::f32(inner) => OrdEqValue::f32(OrderedFloat(*inner)),
            Value::f64(inner) => OrdEqValue::f64(OrderedFloat(*inner)),
            Value::String(inner) => OrdEqValue::String(inner),
            Value::StructValue(inner) => OrdEqValue::StructValue(inner),
            Value::EnumValue(inner) => OrdEqValue::EnumValue(inner),
            Value::TupleStructValue(inner) => OrdEqValue::TupleStructValue(inner),
            Value::TupleValue(inner) => OrdEqValue::TupleValue(inner),
            Value::List(inner) => OrdEqValue::List(inner),
            Value::Map(inner) => OrdEqValue::Map(inner),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        OrdEqValue::from(self) == OrdEqValue::from(other)
    }
}

impl Eq for Value {}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Value {
    fn cmp(&self, other: &Self) -> Ordering {
        OrdEqValue::from(self).cmp(&OrdEqValue::from(other))
    }
}

impl Reflect for Value {
    fn type_info(&self) -> TypeInfoRoot {
        impl Typed for Value {
            fn build(graph: &mut TypeInfoGraph) -> Id {
                graph.get_or_build_with::<Self, _>(|_graph| TypeInfoNode::Opaque)
            }
        }

        <Self as Typed>::type_info()
    }

    fn as_any(&self) -> &dyn Any {
        match self {
            Value::usize(inner) => inner,
            Value::u8(inner) => inner,
            Value::u16(inner) => inner,
            Value::u32(inner) => inner,
            Value::u64(inner) => inner,
            Value::u128(inner) => inner,
            Value::i8(inner) => inner,
            Value::i16(inner) => inner,
            Value::i32(inner) => inner,
            Value::i64(inner) => inner,
            Value::i128(inner) => inner,
            Value::bool(inner) => inner,
            Value::char(inner) => inner,
            Value::f32(inner) => inner,
            Value::f64(inner) => inner,
            Value::String(inner) => inner,
            Value::StructValue(inner) => inner,
            Value::TupleStructValue(inner) => inner,
            Value::EnumValue(inner) => inner,
            Value::TupleValue(inner) => inner,
            Value::List(inner) => inner,
            Value::Map(inner) => inner,
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        match self {
            Value::usize(inner) => inner,
            Value::u8(inner) => inner,
            Value::u16(inner) => inner,
            Value::u32(inner) => inner,
            Value::u64(inner) => inner,
            Value::u128(inner) => inner,
            Value::i8(inner) => inner,
            Value::i16(inner) => inner,
            Value::i32(inner) => inner,
            Value::i64(inner) => inner,
            Value::i128(inner) => inner,
            Value::bool(inner) => inner,
            Value::char(inner) => inner,
            Value::f32(inner) => inner,
            Value::f64(inner) => inner,
            Value::String(inner) => inner,
            Value::StructValue(inner) => inner,
            Value::TupleStructValue(inner) => inner,
            Value::EnumValue(inner) => inner,
            Value::TupleValue(inner) => inner,
            Value::List(inner) => inner,
            Value::Map(inner) => inner,
        }
    }

    fn as_reflect(&self) -> &dyn Reflect {
        self
    }

    fn as_reflect_mut(&mut self) -> &mut dyn Reflect {
        self
    }

    fn reflect_ref(&self) -> ReflectRef<'_> {
        match self {
            Value::usize(inner) => ReflectRef::Scalar(ScalarRef::from(*inner)),
            Value::u8(inner) => ReflectRef::Scalar(ScalarRef::from(*inner)),
            Value::u16(inner) => ReflectRef::Scalar(ScalarRef::from(*inner)),
            Value::u32(inner) => ReflectRef::Scalar(ScalarRef::from(*inner)),
            Value::u64(inner) => ReflectRef::Scalar(ScalarRef::from(*inner)),
            Value::u128(inner) => ReflectRef::Scalar(ScalarRef::from(*inner)),
            Value::i8(inner) => ReflectRef::Scalar(ScalarRef::from(*inner)),
            Value::i16(inner) => ReflectRef::Scalar(ScalarRef::from(*inner)),
            Value::i32(inner) => ReflectRef::Scalar(ScalarRef::from(*inner)),
            Value::i64(inner) => ReflectRef::Scalar(ScalarRef::from(*inner)),
            Value::i128(inner) => ReflectRef::Scalar(ScalarRef::from(*inner)),
            Value::bool(inner) => ReflectRef::Scalar(ScalarRef::from(*inner)),
            Value::char(inner) => ReflectRef::Scalar(ScalarRef::from(*inner)),
            Value::f32(inner) => ReflectRef::Scalar(ScalarRef::from(*inner)),
            Value::f64(inner) => ReflectRef::Scalar(ScalarRef::from(*inner)),
            Value::String(inner) => ReflectRef::Scalar(ScalarRef::from(inner)),
            Value::StructValue(inner) => ReflectRef::Struct(&**inner),
            Value::EnumValue(inner) => ReflectRef::Enum(&**inner),
            Value::TupleStructValue(inner) => ReflectRef::TupleStruct(inner),
            Value::TupleValue(inner) => ReflectRef::Tuple(inner),
            Value::List(inner) => ReflectRef::List(inner),
            Value::Map(inner) => ReflectRef::Map(inner),
        }
    }

    fn reflect_mut(&mut self) -> ReflectMut<'_> {
        match self {
            Value::usize(inner) => ReflectMut::Scalar(ScalarMut::from(inner)),
            Value::u8(inner) => ReflectMut::Scalar(ScalarMut::from(inner)),
            Value::u16(inner) => ReflectMut::Scalar(ScalarMut::from(inner)),
            Value::u32(inner) => ReflectMut::Scalar(ScalarMut::from(inner)),
            Value::u64(inner) => ReflectMut::Scalar(ScalarMut::from(inner)),
            Value::u128(inner) => ReflectMut::Scalar(ScalarMut::from(inner)),
            Value::i8(inner) => ReflectMut::Scalar(ScalarMut::from(inner)),
            Value::i16(inner) => ReflectMut::Scalar(ScalarMut::from(inner)),
            Value::i32(inner) => ReflectMut::Scalar(ScalarMut::from(inner)),
            Value::i64(inner) => ReflectMut::Scalar(ScalarMut::from(inner)),
            Value::i128(inner) => ReflectMut::Scalar(ScalarMut::from(inner)),
            Value::bool(inner) => ReflectMut::Scalar(ScalarMut::from(inner)),
            Value::char(inner) => ReflectMut::Scalar(ScalarMut::from(inner)),
            Value::f32(inner) => ReflectMut::Scalar(ScalarMut::from(inner)),
            Value::f64(inner) => ReflectMut::Scalar(ScalarMut::from(inner)),
            Value::String(inner) => ReflectMut::Scalar(ScalarMut::from(inner)),
            Value::StructValue(inner) => ReflectMut::Struct(&mut **inner),
            Value::EnumValue(inner) => ReflectMut::Enum(&mut **inner),
            Value::TupleStructValue(inner) => ReflectMut::TupleStruct(inner),
            Value::TupleValue(inner) => ReflectMut::Tuple(inner),
            Value::List(inner) => ReflectMut::List(inner),
            Value::Map(inner) => ReflectMut::Map(inner),
        }
    }

    fn patch(&mut self, value: &dyn Reflect) {
        match self {
            Value::usize(inner) => {
                if let Some(value) = value.downcast_ref::<usize>() {
                    *inner = *value;
                }
            }
            Value::u8(inner) => {
                if let Some(value) = value.downcast_ref::<u8>() {
                    *inner = *value;
                }
            }
            Value::u16(inner) => {
                if let Some(value) = value.downcast_ref::<u16>() {
                    *inner = *value;
                }
            }
            Value::u32(inner) => {
                if let Some(value) = value.downcast_ref::<u32>() {
                    *inner = *value;
                }
            }
            Value::u64(inner) => {
                if let Some(value) = value.downcast_ref::<u64>() {
                    *inner = *value;
                }
            }
            Value::u128(inner) => {
                if let Some(value) = value.downcast_ref::<u128>() {
                    *inner = *value;
                }
            }
            Value::i8(inner) => {
                if let Some(value) = value.downcast_ref::<i8>() {
                    *inner = *value;
                }
            }
            Value::i16(inner) => {
                if let Some(value) = value.downcast_ref::<i16>() {
                    *inner = *value;
                }
            }
            Value::i32(inner) => {
                if let Some(value) = value.downcast_ref::<i32>() {
                    *inner = *value;
                }
            }
            Value::i64(inner) => {
                if let Some(value) = value.downcast_ref::<i64>() {
                    *inner = *value;
                }
            }
            Value::i128(inner) => {
                if let Some(value) = value.downcast_ref::<i128>() {
                    *inner = *value;
                }
            }
            Value::bool(inner) => {
                if let Some(value) = value.downcast_ref::<bool>() {
                    *inner = *value;
                }
            }
            Value::char(inner) => {
                if let Some(value) = value.downcast_ref::<char>() {
                    *inner = *value;
                }
            }
            Value::f32(inner) => {
                if let Some(value) = value.downcast_ref::<f32>() {
                    *inner = *value;
                }
            }
            Value::f64(inner) => {
                if let Some(value) = value.downcast_ref::<f64>() {
                    *inner = *value;
                }
            }
            Value::String(inner) => {
                if let Some(value) = value.downcast_ref::<String>() {
                    *inner = value.clone();
                }
            }
            Value::StructValue(inner) => {
                if let Some(value) = value.downcast_ref::<StructValue>() {
                    *inner = Box::new(value.clone());
                }
            }
            Value::TupleStructValue(inner) => {
                if let Some(value) = value.downcast_ref::<TupleStructValue>() {
                    *inner = value.clone();
                }
            }
            Value::EnumValue(inner) => {
                if let Some(value) = value.downcast_ref::<EnumValue>() {
                    *inner = Box::new(value.clone());
                }
            }
            Value::TupleValue(inner) => {
                if let Some(value) = value.downcast_ref::<TupleValue>() {
                    *inner = value.clone();
                }
            }
            Value::List(inner) => {
                if let Some(value) = value.downcast_ref::<Vec<Value>>() {
                    *inner = value.clone();
                }
            }
            Value::Map(inner) => {
                if let Some(value) = value.downcast_ref::<BTreeMap<Value, Value>>() {
                    *inner = value.clone();
                }
            }
        }
    }

    fn to_value(&self) -> Value {
        self.clone()
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

macro_rules! from_impls {
    (
        $($ident:ident)*
    ) => {
        $(
            impl From<$ident> for Value {
                fn from(value: $ident) -> Self {
                    Value::$ident(value)
                }
            }
        )*
    };
}

impl From<StructValue> for Value {
    fn from(value: StructValue) -> Self {
        Value::StructValue(Box::new(value))
    }
}

impl From<EnumValue> for Value {
    fn from(value: EnumValue) -> Self {
        Value::EnumValue(Box::new(value))
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        value.to_owned().into()
    }
}

from_impls! {
    usize u8 u16 u32 u64 u128
    i8 i16 i32 i64 i128
    f32 f64
    bool char String
    TupleValue TupleStructValue
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn has_small_stack_size() {
        // if we can get the value to be smaller than 32 then that'd be cool
        // but 32 is probably also fine
        assert_eq!(std::mem::size_of::<Value>(), 32);
    }
}
