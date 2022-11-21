use serde::{Deserialize, Serialize};
use speedy::{Readable, Writable};
use std::{any::Any, fmt};

use crate::{tuple::TupleValue, Enum, EnumValue, Reflect, Struct, StructValue, Tuple};

#[derive(Readable, Writable, Serialize, Deserialize, Clone)]
pub struct Value(pub(crate) ValueInner);

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

    fn patch(&mut self, value: &dyn Reflect) {
        self.0.patch(value)
    }

    fn clone_reflect(&self) -> Box<dyn Reflect> {
        Box::new(self.clone())
    }

    fn to_value(&self) -> Value {
        self.clone()
    }

    fn as_tuple(&self) -> Option<&dyn Tuple> {
        self.0.as_tuple()
    }

    fn as_tuple_mut(&mut self) -> Option<&mut dyn Tuple> {
        self.0.as_tuple_mut()
    }

    fn as_struct(&self) -> Option<&dyn Struct> {
        self.0.as_struct()
    }

    fn as_struct_mut(&mut self) -> Option<&mut dyn Struct> {
        self.0.as_struct_mut()
    }

    fn as_enum(&self) -> Option<&dyn Enum> {
        self.0.as_enum()
    }

    fn as_enum_mut(&mut self) -> Option<&mut dyn Enum> {
        self.0.as_enum_mut()
    }

    fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "{:#?}", self)
        } else {
            write!(f, "{:?}", self)
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[allow(non_camel_case_types)]
#[derive(Readable, Writable, Serialize, Deserialize, Debug, Clone)]
pub(crate) enum ValueInner {
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
    TupleValue(TupleValue),
}

impl Reflect for ValueInner {
    fn as_any(&self) -> &dyn Any {
        match self {
            ValueInner::usize(inner) => inner,
            ValueInner::u8(inner) => inner,
            ValueInner::u16(inner) => inner,
            ValueInner::u32(inner) => inner,
            ValueInner::u64(inner) => inner,
            ValueInner::u128(inner) => inner,
            ValueInner::i8(inner) => inner,
            ValueInner::i16(inner) => inner,
            ValueInner::i32(inner) => inner,
            ValueInner::i64(inner) => inner,
            ValueInner::i128(inner) => inner,
            ValueInner::bool(inner) => inner,
            ValueInner::char(inner) => inner,
            ValueInner::f32(inner) => inner,
            ValueInner::f64(inner) => inner,
            ValueInner::String(inner) => inner,
            ValueInner::StructValue(inner) => inner,
            ValueInner::EnumValue(inner) => inner,
            ValueInner::TupleValue(inner) => inner,
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        match self {
            ValueInner::usize(inner) => inner,
            ValueInner::u8(inner) => inner,
            ValueInner::u16(inner) => inner,
            ValueInner::u32(inner) => inner,
            ValueInner::u64(inner) => inner,
            ValueInner::u128(inner) => inner,
            ValueInner::i8(inner) => inner,
            ValueInner::i16(inner) => inner,
            ValueInner::i32(inner) => inner,
            ValueInner::i64(inner) => inner,
            ValueInner::i128(inner) => inner,
            ValueInner::bool(inner) => inner,
            ValueInner::char(inner) => inner,
            ValueInner::f32(inner) => inner,
            ValueInner::f64(inner) => inner,
            ValueInner::String(inner) => inner,
            ValueInner::StructValue(inner) => inner,
            ValueInner::EnumValue(inner) => inner,
            ValueInner::TupleValue(inner) => inner,
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
            ValueInner::usize(inner) => {
                if let Some(value) = value.downcast_ref::<usize>() {
                    *inner = *value;
                }
            }
            ValueInner::u8(inner) => {
                if let Some(value) = value.downcast_ref::<u8>() {
                    *inner = *value;
                }
            }
            ValueInner::u16(inner) => {
                if let Some(value) = value.downcast_ref::<u16>() {
                    *inner = *value;
                }
            }
            ValueInner::u32(inner) => {
                if let Some(value) = value.downcast_ref::<u32>() {
                    *inner = *value;
                }
            }
            ValueInner::u64(inner) => {
                if let Some(value) = value.downcast_ref::<u64>() {
                    *inner = *value;
                }
            }
            ValueInner::u128(inner) => {
                if let Some(value) = value.downcast_ref::<u128>() {
                    *inner = *value;
                }
            }
            ValueInner::i8(inner) => {
                if let Some(value) = value.downcast_ref::<i8>() {
                    *inner = *value;
                }
            }
            ValueInner::i16(inner) => {
                if let Some(value) = value.downcast_ref::<i16>() {
                    *inner = *value;
                }
            }
            ValueInner::i32(inner) => {
                if let Some(value) = value.downcast_ref::<i32>() {
                    *inner = *value;
                }
            }
            ValueInner::i64(inner) => {
                if let Some(value) = value.downcast_ref::<i64>() {
                    *inner = *value;
                }
            }
            ValueInner::i128(inner) => {
                if let Some(value) = value.downcast_ref::<i128>() {
                    *inner = *value;
                }
            }
            ValueInner::bool(inner) => {
                if let Some(value) = value.downcast_ref::<bool>() {
                    *inner = *value;
                }
            }
            ValueInner::char(inner) => {
                if let Some(value) = value.downcast_ref::<char>() {
                    *inner = *value;
                }
            }
            ValueInner::f32(inner) => {
                if let Some(value) = value.downcast_ref::<f32>() {
                    *inner = *value;
                }
            }
            ValueInner::f64(inner) => {
                if let Some(value) = value.downcast_ref::<f64>() {
                    *inner = *value;
                }
            }
            ValueInner::String(inner) => {
                if let Some(value) = value.downcast_ref::<String>() {
                    *inner = value.clone();
                }
            }
            ValueInner::StructValue(inner) => {
                if let Some(value) = value.downcast_ref::<StructValue>() {
                    *inner = Box::new(value.clone());
                }
            }
            ValueInner::EnumValue(inner) => {
                if let Some(value) = value.downcast_ref::<EnumValue>() {
                    *inner = Box::new(value.clone());
                }
            }
            ValueInner::TupleValue(inner) => {
                if let Some(value) = value.downcast_ref::<TupleValue>() {
                    *inner = value.clone();
                }
            }
        }
    }

    fn to_value(&self) -> Value {
        Value(self.clone())
    }

    fn clone_reflect(&self) -> Box<dyn Reflect> {
        Box::new(self.clone())
    }

    fn as_tuple(&self) -> Option<&dyn Tuple> {
        if let ValueInner::TupleValue(value) = self {
            Some(value)
        } else {
            None
        }
    }

    fn as_tuple_mut(&mut self) -> Option<&mut dyn Tuple> {
        if let ValueInner::TupleValue(value) = self {
            Some(&mut *value)
        } else {
            None
        }
    }

    fn as_struct(&self) -> Option<&dyn Struct> {
        if let ValueInner::StructValue(value) = self {
            Some(&**value)
        } else {
            None
        }
    }

    fn as_struct_mut(&mut self) -> Option<&mut dyn Struct> {
        if let ValueInner::StructValue(value) = self {
            Some(&mut **value)
        } else {
            None
        }
    }

    fn as_enum(&self) -> Option<&dyn Enum> {
        if let ValueInner::EnumValue(value) = self {
            Some(&**value)
        } else {
            None
        }
    }

    fn as_enum_mut(&mut self) -> Option<&mut dyn Enum> {
        if let ValueInner::EnumValue(value) = self {
            Some(&mut **value)
        } else {
            None
        }
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
                    Self(ValueInner::$ident(value))
                }
            }
        )*
    };
}

impl From<StructValue> for Value {
    fn from(value: StructValue) -> Self {
        Self(ValueInner::StructValue(Box::new(value)))
    }
}

impl From<EnumValue> for Value {
    fn from(value: EnumValue) -> Self {
        Self(ValueInner::EnumValue(Box::new(value)))
    }
}

from_impls! {
    usize u8 u16 u32 u64 u128
    i8 i16 i32 i64 i128
    f32 f64
    bool char String
    TupleValue
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
