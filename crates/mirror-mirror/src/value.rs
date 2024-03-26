use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;
use core::cmp::Ordering;
use core::fmt;
use core::hash::Hash;
use core::hash::Hasher;

use tiny_ordered_float::{OrderedF32, OrderedF64};

use crate::enum_::EnumValue;
use crate::struct_::StructValue;
use crate::tuple::TupleValue;
use crate::tuple_struct::TupleStructValue;
use crate::type_info::graph::NodeId;
use crate::type_info::graph::OpaqueNode;
use crate::type_info::graph::TypeGraph;
use crate::DescribeType;
use crate::FromReflect;
use crate::Reflect;
use crate::ReflectMut;
use crate::ReflectOwned;
use crate::ReflectRef;
use crate::ScalarMut;
use crate::ScalarOwned;
use crate::ScalarRef;
use crate::TypeDescriptor;

/// A type erased value type.
///
/// Constructed with [`Reflect::to_value`].
#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "speedy", derive(speedy::Readable, speedy::Writable))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
#[derive(Eq, PartialEq, PartialOrd, Ord, Hash)]
enum OrdEqHashValue<'a> {
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
    f32(OrderedF32),
    f64(OrderedF64),
    String(&'a str),
    StructValue(&'a StructValue),
    EnumValue(&'a EnumValue),
    TupleStructValue(&'a TupleStructValue),
    TupleValue(&'a TupleValue),
    List(&'a [Value]),
    Map(&'a BTreeMap<Value, Value>),
}

impl<'a> From<&'a Value> for OrdEqHashValue<'a> {
    fn from(value: &'a Value) -> Self {
        match value {
            Value::usize(inner) => OrdEqHashValue::usize(*inner),
            Value::u8(inner) => OrdEqHashValue::u8(*inner),
            Value::u16(inner) => OrdEqHashValue::u16(*inner),
            Value::u32(inner) => OrdEqHashValue::u32(*inner),
            Value::u64(inner) => OrdEqHashValue::u64(*inner),
            Value::u128(inner) => OrdEqHashValue::u128(*inner),
            Value::i8(inner) => OrdEqHashValue::i8(*inner),
            Value::i16(inner) => OrdEqHashValue::i16(*inner),
            Value::i32(inner) => OrdEqHashValue::i32(*inner),
            Value::i64(inner) => OrdEqHashValue::i64(*inner),
            Value::i128(inner) => OrdEqHashValue::i128(*inner),
            Value::bool(inner) => OrdEqHashValue::bool(*inner),
            Value::char(inner) => OrdEqHashValue::char(*inner),
            Value::f32(inner) => OrdEqHashValue::f32(OrderedF32(*inner)),
            Value::f64(inner) => OrdEqHashValue::f64(OrderedF64(*inner)),
            Value::String(inner) => OrdEqHashValue::String(inner),
            Value::StructValue(inner) => OrdEqHashValue::StructValue(inner),
            Value::EnumValue(inner) => OrdEqHashValue::EnumValue(inner),
            Value::TupleStructValue(inner) => OrdEqHashValue::TupleStructValue(inner),
            Value::TupleValue(inner) => OrdEqHashValue::TupleValue(inner),
            Value::List(inner) => OrdEqHashValue::List(inner),
            Value::Map(inner) => OrdEqHashValue::Map(inner),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        OrdEqHashValue::from(self) == OrdEqHashValue::from(other)
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
        OrdEqHashValue::from(self).cmp(&OrdEqHashValue::from(other))
    }
}

impl Hash for Value {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        OrdEqHashValue::from(self).hash(state);
    }
}

macro_rules! for_each_variant {
    ($self:expr, $inner:ident => $expr:expr) => {
        match $self {
            Value::usize($inner) => $expr,
            Value::u8($inner) => $expr,
            Value::u16($inner) => $expr,
            Value::u32($inner) => $expr,
            Value::u64($inner) => $expr,
            Value::u128($inner) => $expr,
            Value::i8($inner) => $expr,
            Value::i16($inner) => $expr,
            Value::i32($inner) => $expr,
            Value::i64($inner) => $expr,
            Value::i128($inner) => $expr,
            Value::bool($inner) => $expr,
            Value::char($inner) => $expr,
            Value::f32($inner) => $expr,
            Value::f64($inner) => $expr,
            Value::String($inner) => $expr,
            Value::StructValue($inner) => $expr,
            Value::TupleStructValue($inner) => $expr,
            Value::EnumValue($inner) => $expr,
            Value::TupleValue($inner) => $expr,
            Value::List($inner) => $expr,
            Value::Map($inner) => $expr,
        }
    };
}

impl DescribeType for Value {
    fn build(graph: &mut TypeGraph) -> NodeId {
        graph.get_or_build_node_with::<Self, _>(|graph| {
            OpaqueNode::new::<Self>(Default::default(), graph)
        })
    }
}

impl Reflect for Value {
    fn type_descriptor(&self) -> alloc::borrow::Cow<'static, TypeDescriptor> {
        <Self as DescribeType>::type_descriptor()
    }

    fn as_any(&self) -> &dyn Any {
        for_each_variant!(self, inner => inner)
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        for_each_variant!(self, inner => inner)
    }

    fn as_reflect(&self) -> &dyn Reflect {
        for_each_variant!(self, inner => inner)
    }

    fn as_reflect_mut(&mut self) -> &mut dyn Reflect {
        for_each_variant!(self, inner => inner)
    }

    fn reflect_owned(self: Box<Self>) -> ReflectOwned {
        match *self {
            Value::usize(inner) => ReflectOwned::Scalar(ScalarOwned::from(inner)),
            Value::u8(inner) => ReflectOwned::Scalar(ScalarOwned::from(inner)),
            Value::u16(inner) => ReflectOwned::Scalar(ScalarOwned::from(inner)),
            Value::u32(inner) => ReflectOwned::Scalar(ScalarOwned::from(inner)),
            Value::u64(inner) => ReflectOwned::Scalar(ScalarOwned::from(inner)),
            Value::u128(inner) => ReflectOwned::Scalar(ScalarOwned::from(inner)),
            Value::i8(inner) => ReflectOwned::Scalar(ScalarOwned::from(inner)),
            Value::i16(inner) => ReflectOwned::Scalar(ScalarOwned::from(inner)),
            Value::i32(inner) => ReflectOwned::Scalar(ScalarOwned::from(inner)),
            Value::i64(inner) => ReflectOwned::Scalar(ScalarOwned::from(inner)),
            Value::i128(inner) => ReflectOwned::Scalar(ScalarOwned::from(inner)),
            Value::bool(inner) => ReflectOwned::Scalar(ScalarOwned::from(inner)),
            Value::char(inner) => ReflectOwned::Scalar(ScalarOwned::from(inner)),
            Value::f32(inner) => ReflectOwned::Scalar(ScalarOwned::from(inner)),
            Value::f64(inner) => ReflectOwned::Scalar(ScalarOwned::from(inner)),
            Value::String(inner) => ReflectOwned::Scalar(ScalarOwned::from(inner)),
            Value::StructValue(inner) => ReflectOwned::Struct(inner),
            Value::EnumValue(inner) => ReflectOwned::Enum(inner),
            Value::TupleStructValue(inner) => ReflectOwned::TupleStruct(Box::new(inner)),
            Value::TupleValue(inner) => ReflectOwned::Tuple(Box::new(inner)),
            Value::List(inner) => ReflectOwned::List(Box::new(inner)),
            Value::Map(inner) => ReflectOwned::Map(Box::new(inner)),
        }
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
        for_each_variant!(self, inner => inner.patch(value))
    }

    fn to_value(&self) -> Value {
        self.clone()
    }

    fn clone_reflect(&self) -> Box<dyn Reflect> {
        Box::new(self.clone())
    }

    fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "{self:#?}")
        } else {
            write!(f, "{self:?}")
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

impl From<&dyn Reflect> for Value {
    fn from(reflect: &dyn Reflect) -> Self {
        reflect.to_value()
    }
}

from_impls! {
    usize u8 u16 u32 u64 u128
    i8 i16 i32 i64 i128
    f32 f64
    bool char String
    TupleValue TupleStructValue
}
