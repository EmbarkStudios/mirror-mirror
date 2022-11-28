use serde::{Deserialize, Serialize};
use speedy::{Readable, Writable};
use std::{any::Any, fmt};

use crate::{
    tuple::TupleValue, Enum, EnumValue, FromReflect, List, Reflect, Struct, StructValue, Tuple,
    TupleStruct, TupleStructValue,
};

#[non_exhaustive]
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
    ListValue(Vec<Value>),
}

impl Reflect for Value {
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
            Value::ListValue(inner) => inner,
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
            Value::ListValue(inner) => inner,
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
            Value::ListValue(inner) => {
                if let Some(value) = value.downcast_ref::<Vec<Value>>() {
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

    fn as_tuple(&self) -> Option<&dyn Tuple> {
        if let Value::TupleValue(value) = self {
            Some(value)
        } else {
            None
        }
    }

    fn as_tuple_mut(&mut self) -> Option<&mut dyn Tuple> {
        if let Value::TupleValue(value) = self {
            Some(&mut *value)
        } else {
            None
        }
    }

    fn as_struct(&self) -> Option<&dyn Struct> {
        if let Value::StructValue(value) = self {
            Some(&**value)
        } else {
            None
        }
    }

    fn as_struct_mut(&mut self) -> Option<&mut dyn Struct> {
        if let Value::StructValue(value) = self {
            Some(&mut **value)
        } else {
            None
        }
    }

    fn as_tuple_struct(&self) -> Option<&dyn TupleStruct> {
        if let Value::TupleStructValue(value) = self {
            Some(value)
        } else {
            None
        }
    }

    fn as_tuple_struct_mut(&mut self) -> Option<&mut dyn TupleStruct> {
        if let Value::TupleStructValue(value) = self {
            Some(value)
        } else {
            None
        }
    }

    fn as_enum(&self) -> Option<&dyn Enum> {
        if let Value::EnumValue(value) = self {
            Some(&**value)
        } else {
            None
        }
    }

    fn as_enum_mut(&mut self) -> Option<&mut dyn Enum> {
        if let Value::EnumValue(value) = self {
            Some(&mut **value)
        } else {
            None
        }
    }

    fn as_list(&self) -> Option<&dyn List> {
        if let Value::ListValue(value) = self {
            Some(&*value)
        } else {
            None
        }
    }

    fn as_list_mut(&mut self) -> Option<&mut dyn List> {
        if let Value::ListValue(value) = self {
            Some(&mut *value)
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

impl From<Vec<Value>> for Value {
    fn from(value: Vec<Value>) -> Self {
        Value::ListValue(value)
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        value.to_owned().into()
    }
}

impl FromReflect for Value {
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
        Some(reflect.to_value())
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
