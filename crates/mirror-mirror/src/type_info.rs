use std::any::type_name;
use std::collections::BTreeMap;
use std::collections::HashMap;

use serde::Deserialize;
use serde::Serialize;
use speedy::Readable;
use speedy::Writable;

use crate::Enum;
use crate::EnumValue;
use crate::FromReflect;
use crate::List;
use crate::Map;
use crate::Reflect;
use crate::Struct;
use crate::StructValue;
use crate::Tuple;
use crate::TupleStruct;
use crate::TupleStructValue;
use crate::TupleValue;
use crate::Value;

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

impl TypeInfo {
    pub fn get_meta(&self, key: &str) -> Option<&dyn Reflect> {
        match self {
            TypeInfo::Struct(x) => x.as_ref().and_then(|x| x.get_meta(key)),
            TypeInfo::TupleStruct(x) => x.as_ref().and_then(|x| x.get_meta(key)),
            TypeInfo::Enum(x) => x.as_ref().and_then(|x| x.get_meta(key)),
            TypeInfo::Tuple(_)
            | TypeInfo::List(_)
            | TypeInfo::Map(_)
            | TypeInfo::Scalar(_)
            | TypeInfo::Value => None,
        }
    }
}

macro_rules! from_impl {
    ($variant:ident(Option<$ty:ty>)) => {
        impl From<$ty> for TypeInfo {
            fn from(value: $ty) -> Self {
                Self::$variant(Some(value))
            }
        }
    };

    ($variant:ident($ty:ty)) => {
        impl From<$ty> for TypeInfo {
            fn from(value: $ty) -> Self {
                Self::$variant(value)
            }
        }
    };
}

from_impl! { Struct(Option<StructInfo>) }
from_impl! { TupleStruct(Option<TupleStructInfo>) }
from_impl! { Tuple(Option<TupleInfo>) }
from_impl! { Enum(Option<EnumInfo>) }
from_impl! { List(ListInfo) }
from_impl! { Map(MapInfo) }
from_impl! { Scalar(ScalarInfo) }

pub type Meta = HashMap<String, Value>;

#[derive(Debug, Clone, Serialize, Deserialize, Readable, Writable)]
pub struct StructInfo {
    type_name: String,
    fields: Vec<NamedField>,
    meta: Meta,
}

impl StructInfo {
    pub fn new<T>(fields: &[NamedField], meta: Meta) -> Self
    where
        T: Struct,
    {
        Self {
            type_name: type_name::<T>().to_owned(),
            fields: fields.to_owned(),
            meta,
        }
    }

    pub fn type_name(&self) -> &str {
        &self.type_name
    }

    pub fn fields(&self) -> &[NamedField] {
        &self.fields
    }

    pub fn get_meta(&self, key: &str) -> Option<&dyn Reflect> {
        Some(self.meta.get(key)?.as_reflect())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Readable, Writable)]
pub struct NamedField {
    name: String,
    type_name: String,
    meta: Meta,
}

impl NamedField {
    pub fn new<T>(name: &'static str, meta: Meta) -> Self
    where
        T: Reflect,
    {
        Self {
            name: name.to_owned(),
            type_name: type_name::<T>().to_owned(),
            meta,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn type_name(&self) -> &str {
        &self.type_name
    }

    pub fn get_meta(&self, key: &str) -> Option<&dyn Reflect> {
        Some(self.meta.get(key)?.as_reflect())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Readable, Writable)]
pub struct TupleStructInfo {
    type_name: String,
    fields: Vec<UnnamedField>,
    meta: Meta,
}

impl TupleStructInfo {
    pub fn new<T>(fields: &[UnnamedField], meta: Meta) -> Self
    where
        T: TupleStruct,
    {
        Self {
            type_name: type_name::<T>().to_owned(),
            fields: fields.to_owned(),
            meta,
        }
    }

    pub fn type_name(&self) -> &str {
        &self.type_name
    }

    pub fn fields(&self) -> &[UnnamedField] {
        &self.fields
    }

    pub fn get_meta(&self, key: &str) -> Option<&dyn Reflect> {
        Some(self.meta.get(key)?.as_reflect())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Readable, Writable)]
pub struct TupleInfo {
    type_name: String,
    elements: Vec<UnnamedField>,
}

impl TupleInfo {
    pub fn new<T>(elements: &[UnnamedField]) -> Self
    where
        T: Tuple,
    {
        Self {
            type_name: type_name::<T>().to_owned(),
            elements: elements.to_owned(),
        }
    }

    pub fn type_name(&self) -> &str {
        &self.type_name
    }

    pub fn elements(&self) -> &[UnnamedField] {
        &self.elements
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Readable, Writable)]
pub struct UnnamedField {
    type_name: String,
    meta: Meta,
}

impl UnnamedField {
    pub fn new<T>(meta: Meta) -> Self
    where
        T: Reflect,
    {
        Self {
            type_name: type_name::<T>().to_owned(),
            meta,
        }
    }

    pub fn type_name(&self) -> &str {
        &self.type_name
    }

    pub fn get_meta(&self, key: &str) -> Option<&dyn Reflect> {
        Some(self.meta.get(key)?.as_reflect())
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
    meta: Meta,
}

impl EnumInfo {
    pub fn new<T>(variants: &[VariantInfo], meta: Meta) -> Self
    where
        T: Enum,
    {
        Self {
            type_name: type_name::<T>().to_owned(),
            variants: variants.to_owned(),
            meta,
        }
    }

    pub fn type_name(&self) -> &str {
        &self.type_name
    }

    pub fn get_meta(&self, key: &str) -> Option<&dyn Reflect> {
        Some(self.meta.get(key)?.as_reflect())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Readable, Writable)]
pub enum VariantInfo {
    Struct(StructVariantInfo),
    Tuple(TupleVariantInfo),
    Unit(UnitVariantInfo),
}

impl VariantInfo {
    pub fn get_meta(&self, key: &str) -> Option<&dyn Reflect> {
        match self {
            VariantInfo::Struct(x) => x.get_meta(key),
            VariantInfo::Tuple(x) => x.get_meta(key),
            VariantInfo::Unit(x) => x.get_meta(key),
        }
    }
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
    meta: Meta,
}

impl StructVariantInfo {
    pub fn new(name: &'static str, fields: &[NamedField], meta: Meta) -> Self {
        Self {
            name: name.to_owned(),
            fields: fields.to_owned(),
            meta,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn fields(&self) -> &[NamedField] {
        &self.fields
    }

    pub fn get_meta(&self, key: &str) -> Option<&dyn Reflect> {
        Some(self.meta.get(key)?.as_reflect())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Readable, Writable)]
pub struct TupleVariantInfo {
    name: String,
    fields: Vec<UnnamedField>,
    meta: Meta,
}

impl TupleVariantInfo {
    pub fn new(name: &'static str, fields: &[UnnamedField], meta: Meta) -> Self {
        Self {
            name: name.to_owned(),
            fields: fields.to_owned(),
            meta,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn fields(&self) -> &[UnnamedField] {
        &self.fields
    }

    pub fn get_meta(&self, key: &str) -> Option<&dyn Reflect> {
        Some(self.meta.get(key)?.as_reflect())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Readable, Writable)]
pub struct UnitVariantInfo {
    name: String,
    meta: Meta,
}

impl UnitVariantInfo {
    pub fn new(name: &'static str, meta: Meta) -> Self {
        Self {
            name: name.to_owned(),
            meta,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn get_meta(&self, key: &str) -> Option<&dyn Reflect> {
        Some(self.meta.get(key)?.as_reflect())
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
