use std::{any::type_name, collections::BTreeMap};

use serde::{Deserialize, Serialize};
use speedy::{Readable, Writable};

use crate::{
    Enum, EnumValue, FromReflect, List, Map, Reflect, Struct, StructValue, Tuple, TupleStruct,
    TupleStructValue, TupleValue, Value,
};

#[derive(Debug, Clone, Serialize, Deserialize, Readable, Writable)]
pub enum TypeInfo {
    Struct(Option<StructInfo>),
    TupleStruct(Option<TupleStructInfo>),
    Tuple(Option<TupleInfo>),
    Enum(Option<EnumInfo>),
    List(ListInfo),
    Map(MapInfo),
    Scalar(ScalarInfo),
    Value,
}

macro_rules! from_impl {
    ($variant:ident($ty:ty)) => {
        impl From<$ty> for TypeInfo {
            fn from(value: $ty) -> Self {
                Self::$variant(Some(value))
            }
        }
    };
}

from_impl! { Struct(StructInfo) }
from_impl! { TupleStruct(TupleStructInfo) }
from_impl! { Tuple(TupleInfo) }
from_impl! { Enum(EnumInfo) }

impl From<ListInfo> for TypeInfo {
    fn from(value: ListInfo) -> Self {
        Self::List(value)
    }
}

impl From<MapInfo> for TypeInfo {
    fn from(value: MapInfo) -> Self {
        Self::Map(value)
    }
}

impl From<ScalarInfo> for TypeInfo {
    fn from(value: ScalarInfo) -> Self {
        Self::Scalar(value)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Readable, Writable)]
pub struct StructInfo {
    type_name: String,
    fields: Vec<NamedField>,
}

impl StructInfo {
    pub fn new<T>(fields: &[NamedField]) -> Self
    where
        T: Struct,
    {
        Self {
            type_name: type_name::<T>().to_owned(),
            fields: fields.to_owned(),
        }
    }

    pub fn type_name(&self) -> &str {
        &self.type_name
    }

    pub fn fields(&self) -> &[NamedField] {
        &self.fields
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Readable, Writable)]
pub struct NamedField {
    name: String,
    type_name: String,
}

impl NamedField {
    pub fn new<T>(name: &'static str) -> Self
    where
        T: Reflect,
    {
        Self {
            name: name.to_owned(),
            type_name: type_name::<T>().to_owned(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn type_name(&self) -> &str {
        &self.type_name
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Readable, Writable)]
pub struct TupleStructInfo {
    type_name: String,
    fields: Vec<UnnamedField>,
}

impl TupleStructInfo {
    pub fn new<T>(fields: &[UnnamedField]) -> Self
    where
        T: TupleStruct,
    {
        Self {
            type_name: type_name::<T>().to_owned(),
            fields: fields.to_owned(),
        }
    }

    pub fn type_name(&self) -> &str {
        &self.type_name
    }

    pub fn fields(&self) -> &[UnnamedField] {
        &self.fields
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Readable, Writable)]
pub struct TupleInfo {
    type_name: String,
    fields: Vec<UnnamedField>,
}

impl TupleInfo {
    pub fn new<T>(fields: &[UnnamedField]) -> Self
    where
        T: Tuple,
    {
        Self {
            type_name: type_name::<T>().to_owned(),
            fields: fields.to_owned(),
        }
    }

    pub fn type_name(&self) -> &str {
        &self.type_name
    }

    pub fn fields(&self) -> &[UnnamedField] {
        &self.fields
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Readable, Writable)]
pub struct UnnamedField {
    type_name: String,
}

impl UnnamedField {
    pub fn new<T>() -> Self
    where
        T: Reflect,
    {
        Self {
            type_name: type_name::<T>().to_owned(),
        }
    }

    pub fn type_name(&self) -> &str {
        &self.type_name
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Readable, Writable)]
pub struct ListInfo {
    type_name: String,
    element_type_name: String,
}

impl ListInfo {
    pub fn new<T, I>() -> Self
    where
        T: List,
        I: Reflect,
    {
        Self {
            type_name: type_name::<T>().to_owned(),
            element_type_name: type_name::<I>().to_owned(),
        }
    }

    pub fn type_name(&self) -> &str {
        &self.type_name
    }

    pub fn element_type_name(&self) -> &str {
        &self.element_type_name
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Readable, Writable)]
pub struct MapInfo {
    type_name: String,
    key_type_name: String,
    value_type_name: String,
}

impl MapInfo {
    pub fn new<T, K, V>() -> Self
    where
        T: Map,
        K: Reflect,
        V: Reflect,
    {
        Self {
            type_name: type_name::<T>().to_owned(),
            key_type_name: type_name::<K>().to_owned(),
            value_type_name: type_name::<V>().to_owned(),
        }
    }

    pub fn type_name(&self) -> &str {
        &self.type_name
    }

    pub fn key_type_name(&self) -> &str {
        &self.key_type_name
    }

    pub fn element_type_name(&self) -> &str {
        &self.value_type_name
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Readable, Writable)]
pub struct EnumInfo {
    type_name: String,
    variants: Vec<VariantInfo>,
}

impl EnumInfo {
    pub fn new<T>(variants: &[VariantInfo]) -> Self
    where
        T: Enum,
    {
        Self {
            type_name: type_name::<T>().to_owned(),
            variants: variants.to_owned(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Readable, Writable)]
pub enum VariantInfo {
    Struct(StructVariantInfo),
    Tuple(TupleVariantInfo),
    Unit(UnitVariantInfo),
}

impl From<StructVariantInfo> for VariantInfo {
    fn from(info: StructVariantInfo) -> Self {
        Self::Struct(info)
    }
}

impl From<TupleVariantInfo> for VariantInfo {
    fn from(info: TupleVariantInfo) -> Self {
        Self::Tuple(info)
    }
}

impl From<UnitVariantInfo> for VariantInfo {
    fn from(info: UnitVariantInfo) -> Self {
        Self::Unit(info)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Readable, Writable)]
pub struct StructVariantInfo {
    name: String,
    fields: Vec<NamedField>,
}

impl StructVariantInfo {
    pub fn new(name: &'static str, fields: &[NamedField]) -> Self {
        Self {
            name: name.to_owned(),
            fields: fields.to_owned(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn fields(&self) -> &[NamedField] {
        &self.fields
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Readable, Writable)]
pub struct TupleVariantInfo {
    name: String,
    fields: Vec<UnnamedField>,
}

impl TupleVariantInfo {
    pub fn new(name: &'static str, fields: &[UnnamedField]) -> Self {
        Self {
            name: name.to_owned(),
            fields: fields.to_owned(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn fields(&self) -> &[UnnamedField] {
        &self.fields
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Readable, Writable)]
pub struct UnitVariantInfo {
    name: String,
}

impl UnitVariantInfo {
    pub fn new(name: &'static str) -> Self {
        Self {
            name: name.to_owned(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Readable, Writable)]
#[allow(non_camel_case_types)]
pub enum ScalarInfo {
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
}

pub trait Typed: Reflect {
    fn type_info() -> TypeInfo;
}

impl Typed for Value {
    fn type_info() -> TypeInfo {
        TypeInfo::Value
    }
}

impl Typed for StructValue {
    fn type_info() -> TypeInfo {
        TypeInfo::Struct(None)
    }
}

impl Typed for TupleStructValue {
    fn type_info() -> TypeInfo {
        TypeInfo::TupleStruct(None)
    }
}

impl Typed for TupleValue {
    fn type_info() -> TypeInfo {
        TypeInfo::Tuple(None)
    }
}

impl Typed for EnumValue {
    fn type_info() -> TypeInfo {
        TypeInfo::Enum(None)
    }
}

impl<T> Typed for Vec<T>
where
    T: FromReflect,
{
    fn type_info() -> TypeInfo {
        ListInfo::new::<Self, T>().into()
    }
}

impl<K, V> Typed for BTreeMap<K, V>
where
    K: FromReflect + Ord,
    V: FromReflect,
{
    fn type_info() -> TypeInfo {
        MapInfo::new::<Self, K, V>().into()
    }
}
