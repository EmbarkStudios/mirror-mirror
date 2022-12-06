use std::collections::BTreeMap;

use serde::Deserialize;
use serde::Serialize;
use speedy::Readable;
use speedy::Writable;

use graph::*;

use crate::enum_::EnumValue;
use crate::key_path::Key;
use crate::key_path::KeyPath;
use crate::struct_::StructValue;
use crate::struct_::TupleStructValue;
use crate::tuple::TupleValue;
use crate::Reflect;
use crate::Value;

pub mod graph;

pub trait Typed: 'static {
    fn type_info() -> TypeInfoRoot {
        let mut graph = TypeInfoGraph::default();
        let id = Self::build(&mut graph);
        TypeInfoRoot { root: id, graph }
    }

    fn build(graph: &mut TypeInfoGraph) -> Id;
}

pub trait GetTypedPath<'a> {
    fn at_typed(self, key_path: KeyPath) -> Option<TypeInfoAtPath<'a>>;
}

#[derive(Debug, Clone, Serialize, Deserialize, Writable, Readable)]
pub struct TypeInfoRoot {
    root: Id,
    graph: TypeInfoGraph,
}

impl TypeInfoRoot {
    pub fn type_(&self) -> TypeInfo<'_> {
        TypeInfo::new(self.root, &self.graph)
    }
}

impl<'a> GetTypedPath<'a> for &'a TypeInfoRoot {
    fn at_typed(self, key_path: KeyPath) -> Option<TypeInfoAtPath<'a>> {
        self.type_().at_typed(key_path)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum TypeInfo<'a> {
    Struct(StructInfo<'a>),
    TupleStruct(TupleStructInfo<'a>),
    Tuple(TupleInfo<'a>),
    Enum(EnumInfo<'a>),
    List(ListInfo<'a>),
    Array(ArrayInfo<'a>),
    Map(MapInfo<'a>),
    Scalar(ScalarInfo),
    Opaque(OpaqueInfo<'a>),
}

impl<'a> TypeInfo<'a> {
    fn new(id: Id, graph: &'a TypeInfoGraph) -> Self {
        match graph.get(id) {
            TypeInfoNode::Struct(node) => {
                let node = StructInfo { node, graph };
                TypeInfo::Struct(node)
            }
            TypeInfoNode::TupleStruct(node) => {
                let node = TupleStructInfo { node, graph };
                TypeInfo::TupleStruct(node)
            }
            TypeInfoNode::Tuple(node) => {
                let node = TupleInfo { node, graph };
                TypeInfo::Tuple(node)
            }
            TypeInfoNode::Enum(node) => {
                let node = EnumInfo { node, graph };
                TypeInfo::Enum(node)
            }
            TypeInfoNode::List(node) => {
                let node = ListInfo { node, graph };
                TypeInfo::List(node)
            }
            TypeInfoNode::Array(node) => {
                let node = ArrayInfo { node, graph };
                TypeInfo::Array(node)
            }
            TypeInfoNode::Map(node) => {
                let node = MapInfo { node, graph };
                TypeInfo::Map(node)
            }
            TypeInfoNode::Scalar(scalar) => {
                let node = match scalar {
                    ScalarInfoNode::usize => ScalarInfo::usize,
                    ScalarInfoNode::u8 => ScalarInfo::u8,
                    ScalarInfoNode::u16 => ScalarInfo::u16,
                    ScalarInfoNode::u32 => ScalarInfo::u32,
                    ScalarInfoNode::u64 => ScalarInfo::u64,
                    ScalarInfoNode::u128 => ScalarInfo::u128,
                    ScalarInfoNode::i8 => ScalarInfo::i8,
                    ScalarInfoNode::i16 => ScalarInfo::i16,
                    ScalarInfoNode::i32 => ScalarInfo::i32,
                    ScalarInfoNode::i64 => ScalarInfo::i64,
                    ScalarInfoNode::i128 => ScalarInfo::i128,
                    ScalarInfoNode::bool => ScalarInfo::bool,
                    ScalarInfoNode::char => ScalarInfo::char,
                    ScalarInfoNode::f32 => ScalarInfo::f32,
                    ScalarInfoNode::f64 => ScalarInfo::f64,
                    ScalarInfoNode::String => ScalarInfo::String,
                };
                TypeInfo::Scalar(node)
            }
            TypeInfoNode::Opaque(node) => {
                let node = OpaqueInfo { node, graph };
                TypeInfo::Opaque(node)
            }
        }
    }

    pub fn type_name(self) -> &'a str {
        match self {
            TypeInfo::Struct(inner) => inner.type_name(),
            TypeInfo::TupleStruct(inner) => inner.type_name(),
            TypeInfo::Tuple(inner) => inner.type_name(),
            TypeInfo::Enum(inner) => inner.type_name(),
            TypeInfo::List(inner) => inner.type_name(),
            TypeInfo::Array(inner) => inner.type_name(),
            TypeInfo::Map(inner) => inner.type_name(),
            TypeInfo::Scalar(inner) => inner.type_name(),
            TypeInfo::Opaque(inner) => inner.type_name(),
        }
    }

    fn into_type_info_at_path(self) -> TypeInfoAtPath<'a> {
        match self {
            TypeInfo::Struct(inner) => inner.into_type_info_at_path(),
            TypeInfo::TupleStruct(inner) => inner.into_type_info_at_path(),
            TypeInfo::Tuple(inner) => inner.into_type_info_at_path(),
            TypeInfo::Enum(inner) => inner.into_type_info_at_path(),
            TypeInfo::List(inner) => inner.into_type_info_at_path(),
            TypeInfo::Array(inner) => inner.into_type_info_at_path(),
            TypeInfo::Map(inner) => inner.into_type_info_at_path(),
            TypeInfo::Scalar(inner) => inner.into_type_info_at_path(),
            TypeInfo::Opaque(inner) => inner.into_type_info_at_path(),
        }
    }

    pub fn default_value(self) -> Option<Value> {
        let value = match self {
            TypeInfo::Struct(struct_) => {
                let mut value = StructValue::new();
                for field in struct_.fields() {
                    value.set_field(field.name(), field.type_().default_value()?);
                }
                value.to_value()
            }
            TypeInfo::TupleStruct(tuple_struct) => {
                let mut value = TupleStructValue::new();
                for field in tuple_struct.fields() {
                    value.push_field(field.type_().default_value()?);
                }
                value.to_value()
            }
            TypeInfo::Tuple(tuple) => {
                let mut value = TupleValue::new();
                for field in tuple.fields() {
                    value.push_field(field.type_().default_value()?);
                }
                value.to_value()
            }
            TypeInfo::Enum(enum_) => {
                let mut variants = enum_.variants();
                let variant = variants.next()?;
                match variant {
                    VariantInfo::Struct(variant) => {
                        let mut value = EnumValue::new_struct_variant(variant.name());
                        for field in variant.fields() {
                            value.set_struct_field(field.name(), field.type_().default_value()?);
                        }
                        value.to_value()
                    }
                    VariantInfo::Tuple(variant) => {
                        let mut value = EnumValue::new_tuple_variant(variant.name());
                        for field in variant.fields() {
                            value.push_tuple_field(field.type_().default_value()?);
                        }
                        value.to_value()
                    }
                    VariantInfo::Unit(variant) => {
                        EnumValue::new_unit_variant(variant.name()).to_value()
                    }
                }
            }
            TypeInfo::List(_) => Vec::<()>::new().to_value(),
            TypeInfo::Array(_) => <[(); 0] as Reflect>::to_value(&[]),
            TypeInfo::Map(_) => BTreeMap::<(), ()>::new().to_value(),
            TypeInfo::Scalar(scalar) => match scalar {
                ScalarInfo::usize => usize::default().to_value(),
                ScalarInfo::u8 => u8::default().to_value(),
                ScalarInfo::u16 => u16::default().to_value(),
                ScalarInfo::u32 => u32::default().to_value(),
                ScalarInfo::u64 => u64::default().to_value(),
                ScalarInfo::u128 => u128::default().to_value(),
                ScalarInfo::i8 => i8::default().to_value(),
                ScalarInfo::i16 => i16::default().to_value(),
                ScalarInfo::i32 => i32::default().to_value(),
                ScalarInfo::i64 => i64::default().to_value(),
                ScalarInfo::i128 => i128::default().to_value(),
                ScalarInfo::bool => bool::default().to_value(),
                ScalarInfo::char => char::default().to_value(),
                ScalarInfo::f32 => f32::default().to_value(),
                ScalarInfo::f64 => f64::default().to_value(),
                ScalarInfo::String => String::default().to_value(),
            },
            TypeInfo::Opaque(_) => return None,
        };
        Some(value)
    }

    pub fn as_struct(self) -> Option<StructInfo<'a>> {
        match self {
            TypeInfo::Struct(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_tuple_struct(self) -> Option<TupleStructInfo<'a>> {
        match self {
            TypeInfo::TupleStruct(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_tuple(self) -> Option<TupleInfo<'a>> {
        match self {
            TypeInfo::Tuple(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_enum(self) -> Option<EnumInfo<'a>> {
        match self {
            TypeInfo::Enum(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_array(self) -> Option<ArrayInfo<'a>> {
        match self {
            TypeInfo::Array(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_list(self) -> Option<ListInfo<'a>> {
        match self {
            TypeInfo::List(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_map(self) -> Option<MapInfo<'a>> {
        match self {
            TypeInfo::Map(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_scalar(self) -> Option<ScalarInfo> {
        match self {
            TypeInfo::Scalar(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_opaque(self) -> Option<OpaqueInfo<'a>> {
        match self {
            TypeInfo::Opaque(inner) => Some(inner),
            _ => None,
        }
    }
}

impl<'a> GetTypedPath<'a> for TypeInfo<'a> {
    fn at_typed(self, key_path: KeyPath) -> Option<TypeInfoAtPath<'a>> {
        self.into_type_info_at_path().at_typed(key_path)
    }
}

impl<'a> GetTypedPath<'a> for StructInfo<'a> {
    fn at_typed(self, key_path: KeyPath) -> Option<TypeInfoAtPath<'a>> {
        self.into_type_info_at_path().at_typed(key_path)
    }
}

impl<'a> GetTypedPath<'a> for TupleStructInfo<'a> {
    fn at_typed(self, key_path: KeyPath) -> Option<TypeInfoAtPath<'a>> {
        self.into_type_info_at_path().at_typed(key_path)
    }
}

impl<'a> GetTypedPath<'a> for TupleInfo<'a> {
    fn at_typed(self, key_path: KeyPath) -> Option<TypeInfoAtPath<'a>> {
        self.into_type_info_at_path().at_typed(key_path)
    }
}

impl<'a> GetTypedPath<'a> for EnumInfo<'a> {
    fn at_typed(self, key_path: KeyPath) -> Option<TypeInfoAtPath<'a>> {
        self.into_type_info_at_path().at_typed(key_path)
    }
}

impl<'a> GetTypedPath<'a> for ListInfo<'a> {
    fn at_typed(self, key_path: KeyPath) -> Option<TypeInfoAtPath<'a>> {
        self.into_type_info_at_path().at_typed(key_path)
    }
}

impl<'a> GetTypedPath<'a> for MapInfo<'a> {
    fn at_typed(self, key_path: KeyPath) -> Option<TypeInfoAtPath<'a>> {
        self.into_type_info_at_path().at_typed(key_path)
    }
}

impl GetTypedPath<'static> for ScalarInfo {
    fn at_typed(self, key_path: KeyPath) -> Option<TypeInfoAtPath<'static>> {
        self.into_type_info_at_path().at_typed(key_path)
    }
}

impl<'a> GetTypedPath<'a> for VariantInfo<'a> {
    fn at_typed(self, key_path: KeyPath) -> Option<TypeInfoAtPath<'a>> {
        self.into_type_info_at_path().at_typed(key_path)
    }
}

pub trait GetMeta<'a> {
    fn meta(self, key: &str) -> Option<&'a dyn Reflect>;

    fn get_meta<T>(self, key: &str) -> Option<&'a T>
    where
        T: Reflect,
        Self: Sized,
    {
        self.meta(key)?.downcast_ref()
    }

    fn docs(self) -> &'a [String];
}

impl<'a> GetMeta<'a> for TypeInfo<'a> {
    fn meta(self, key: &str) -> Option<&'a dyn Reflect> {
        match self {
            TypeInfo::Struct(inner) => inner.meta(key),
            TypeInfo::TupleStruct(inner) => inner.meta(key),
            TypeInfo::Enum(inner) => inner.meta(key),
            TypeInfo::Opaque(inner) => inner.meta(key),
            TypeInfo::Tuple(_)
            | TypeInfo::List(_)
            | TypeInfo::Array(_)
            | TypeInfo::Map(_)
            | TypeInfo::Scalar(_) => None,
        }
    }

    fn docs(self) -> &'a [String] {
        match self {
            TypeInfo::Struct(inner) => inner.docs(),
            TypeInfo::TupleStruct(inner) => inner.docs(),
            TypeInfo::Enum(inner) => inner.docs(),
            TypeInfo::Tuple(_)
            | TypeInfo::List(_)
            | TypeInfo::Array(_)
            | TypeInfo::Map(_)
            | TypeInfo::Scalar(_)
            | TypeInfo::Opaque(_) => &[],
        }
    }
}

macro_rules! impl_get_meta {
    ($($ident:ident)*) => {
        $(
            impl<'a> GetMeta<'a> for $ident<'a> {
                fn meta(self, key: &str) -> Option<&'a dyn Reflect> {
                    Some(self.node.metadata.get(key)?.as_reflect())
                }

                fn docs(self) -> &'a [String] {
                    &self.node.docs
                }
            }
        )*
    };
}

impl_get_meta! {
    StructInfo
    TupleStructInfo
    EnumInfo
    StructVariantInfo
    TupleVariantInfo
    UnitVariantInfo
    UnnamedField
    NamedField
}

impl<'a> GetMeta<'a> for OpaqueInfo<'a> {
    fn meta(self, key: &str) -> Option<&'a dyn Reflect> {
        Some(self.node.metadata.get(key)?.as_reflect())
    }

    fn docs(self) -> &'a [String] {
        &[]
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Writable, Readable)]
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

impl ScalarInfo {
    pub fn type_name(self) -> &'static str {
        match self {
            ScalarInfo::usize => "usize",
            ScalarInfo::u8 => "u8",
            ScalarInfo::u16 => "u16",
            ScalarInfo::u32 => "u32",
            ScalarInfo::u64 => "u64",
            ScalarInfo::u128 => "u128",
            ScalarInfo::i8 => "i8",
            ScalarInfo::i16 => "i16",
            ScalarInfo::i32 => "i32",
            ScalarInfo::i64 => "i64",
            ScalarInfo::i128 => "i128",
            ScalarInfo::bool => "bool",
            ScalarInfo::char => "char",
            ScalarInfo::f32 => "f32",
            ScalarInfo::f64 => "f64",
            ScalarInfo::String => "String",
        }
    }

    fn into_type_info_at_path(self) -> TypeInfoAtPath<'static> {
        TypeInfoAtPath::Scalar(self)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct StructInfo<'a> {
    node: &'a StructInfoNode,
    graph: &'a TypeInfoGraph,
}

impl<'a> StructInfo<'a> {
    pub fn type_name(self) -> &'a str {
        &self.node.type_name
    }

    pub fn fields(self) -> impl Iterator<Item = NamedField<'a>> {
        self.node.fields.iter().map(|node| NamedField {
            node,
            graph: self.graph,
        })
    }

    fn into_type_info_at_path(self) -> TypeInfoAtPath<'a> {
        TypeInfoAtPath::Struct(self)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct TupleStructInfo<'a> {
    node: &'a TupleStructInfoNode,
    graph: &'a TypeInfoGraph,
}

impl<'a> TupleStructInfo<'a> {
    pub fn type_name(self) -> &'a str {
        &self.node.type_name
    }

    pub fn fields(self) -> impl Iterator<Item = UnnamedField<'a>> {
        self.node.fields.iter().map(|node| UnnamedField {
            node,
            graph: self.graph,
        })
    }

    fn into_type_info_at_path(self) -> TypeInfoAtPath<'a> {
        TypeInfoAtPath::TupleStruct(self)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct TupleInfo<'a> {
    node: &'a TupleInfoNode,
    graph: &'a TypeInfoGraph,
}

impl<'a> TupleInfo<'a> {
    pub fn type_name(self) -> &'a str {
        &self.node.type_name
    }

    pub fn fields(self) -> impl Iterator<Item = UnnamedField<'a>> {
        self.node.fields.iter().map(|node| UnnamedField {
            node,
            graph: self.graph,
        })
    }

    fn into_type_info_at_path(self) -> TypeInfoAtPath<'a> {
        TypeInfoAtPath::Tuple(self)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct EnumInfo<'a> {
    node: &'a EnumInfoNode,
    graph: &'a TypeInfoGraph,
}

impl<'a> EnumInfo<'a> {
    pub fn type_name(self) -> &'a str {
        &self.node.type_name
    }

    pub fn variants(self) -> impl Iterator<Item = VariantInfo<'a>> {
        self.node.variants.iter().map(|variant| match variant {
            VariantNode::Struct(node) => VariantInfo::Struct(StructVariantInfo {
                node,
                enum_node: self.node,
                graph: self.graph,
            }),
            VariantNode::Tuple(node) => VariantInfo::Tuple(TupleVariantInfo {
                node,
                enum_node: self.node,
                graph: self.graph,
            }),
            VariantNode::Unit(node) => VariantInfo::Unit(UnitVariantInfo {
                node,
                enum_node: self.node,
                graph: self.graph,
            }),
        })
    }

    fn into_type_info_at_path(self) -> TypeInfoAtPath<'a> {
        TypeInfoAtPath::Enum(self)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum VariantInfo<'a> {
    Struct(StructVariantInfo<'a>),
    Tuple(TupleVariantInfo<'a>),
    Unit(UnitVariantInfo<'a>),
}

impl<'a> VariantInfo<'a> {
    pub fn name(self) -> &'a str {
        match self {
            VariantInfo::Struct(inner) => inner.name(),
            VariantInfo::Tuple(inner) => inner.name(),
            VariantInfo::Unit(inner) => inner.name(),
        }
    }

    pub fn fields(self) -> impl Iterator<Item = VariantField<'a>> {
        match self {
            VariantInfo::Struct(inner) => Box::new(inner.fields().map(VariantField::Named))
                as Box<dyn Iterator<Item = VariantField<'a>>>,
            VariantInfo::Tuple(inner) => Box::new(inner.fields().map(VariantField::Unnamed)),
            VariantInfo::Unit(_) => Box::new(std::iter::empty()),
        }
    }

    pub fn enum_type(self) -> EnumInfo<'a> {
        match self {
            VariantInfo::Struct(inner) => inner.enum_type(),
            VariantInfo::Tuple(inner) => inner.enum_type(),
            VariantInfo::Unit(inner) => inner.enum_type(),
        }
    }

    fn into_type_info_at_path(self) -> TypeInfoAtPath<'a> {
        TypeInfoAtPath::Variant(self)
    }
}

impl<'a> GetMeta<'a> for VariantInfo<'a> {
    fn meta(self, key: &str) -> Option<&'a dyn Reflect> {
        match self {
            VariantInfo::Struct(inner) => inner.meta(key),
            VariantInfo::Tuple(inner) => inner.meta(key),
            VariantInfo::Unit(inner) => inner.meta(key),
        }
    }

    fn docs(self) -> &'a [String] {
        match self {
            VariantInfo::Struct(inner) => inner.docs(),
            VariantInfo::Tuple(inner) => inner.docs(),
            VariantInfo::Unit(inner) => inner.docs(),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum VariantField<'a> {
    Named(NamedField<'a>),
    Unnamed(UnnamedField<'a>),
}

impl<'a> VariantField<'a> {
    pub fn type_(self) -> TypeInfo<'a> {
        match self {
            VariantField::Named(inner) => inner.type_(),
            VariantField::Unnamed(inner) => inner.type_(),
        }
    }

    pub fn name(self) -> Option<&'a str> {
        match self {
            VariantField::Named(inner) => Some(inner.name()),
            VariantField::Unnamed(_) => None,
        }
    }
}

impl<'a> GetMeta<'a> for VariantField<'a> {
    fn meta(self, key: &str) -> Option<&'a dyn Reflect> {
        match self {
            VariantField::Named(inner) => inner.meta(key),
            VariantField::Unnamed(inner) => inner.meta(key),
        }
    }

    fn docs(self) -> &'a [String] {
        match self {
            VariantField::Named(inner) => inner.docs(),
            VariantField::Unnamed(inner) => inner.docs(),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct StructVariantInfo<'a> {
    node: &'a StructVariantInfoNode,
    enum_node: &'a EnumInfoNode,
    graph: &'a TypeInfoGraph,
}

impl<'a> StructVariantInfo<'a> {
    pub fn name(self) -> &'a str {
        &self.node.name
    }

    pub fn fields(self) -> impl Iterator<Item = NamedField<'a>> {
        self.node.fields.iter().map(|node| NamedField {
            node,
            graph: self.graph,
        })
    }

    pub fn enum_type(self) -> EnumInfo<'a> {
        EnumInfo {
            node: self.enum_node,
            graph: self.graph,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct TupleVariantInfo<'a> {
    node: &'a TupleVariantInfoNode,
    enum_node: &'a EnumInfoNode,
    graph: &'a TypeInfoGraph,
}

impl<'a> TupleVariantInfo<'a> {
    pub fn name(self) -> &'a str {
        &self.node.name
    }

    pub fn fields(self) -> impl Iterator<Item = UnnamedField<'a>> {
        self.node.fields.iter().map(|node| UnnamedField {
            node,
            graph: self.graph,
        })
    }

    pub fn enum_type(self) -> EnumInfo<'a> {
        EnumInfo {
            node: self.enum_node,
            graph: self.graph,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct UnitVariantInfo<'a> {
    node: &'a UnitVariantInfoNode,
    enum_node: &'a EnumInfoNode,
    graph: &'a TypeInfoGraph,
}

impl<'a> UnitVariantInfo<'a> {
    pub fn name(self) -> &'a str {
        &self.node.name
    }

    pub fn enum_type(self) -> EnumInfo<'a> {
        EnumInfo {
            node: self.enum_node,
            graph: self.graph,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct UnnamedField<'a> {
    node: &'a UnnamedFieldNode,
    graph: &'a TypeInfoGraph,
}

impl<'a> UnnamedField<'a> {
    pub fn type_(self) -> TypeInfo<'a> {
        TypeInfo::new(self.node.id, self.graph)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct NamedField<'a> {
    node: &'a NamedFieldNode,
    graph: &'a TypeInfoGraph,
}

impl<'a> NamedField<'a> {
    pub fn name(self) -> &'a str {
        &self.node.name
    }

    pub fn type_(self) -> TypeInfo<'a> {
        TypeInfo::new(self.node.id, self.graph)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ArrayInfo<'a> {
    node: &'a ArrayInfoNode,
    graph: &'a TypeInfoGraph,
}

impl<'a> ArrayInfo<'a> {
    pub fn type_name(self) -> &'a str {
        &self.node.type_name
    }

    pub fn field_type(self) -> TypeInfo<'a> {
        TypeInfo::new(self.node.field_type_id, self.graph)
    }

    pub fn len(self) -> usize {
        self.node.len
    }

    pub fn is_empty(self) -> bool {
        self.node.len == 0
    }

    fn into_type_info_at_path(self) -> TypeInfoAtPath<'a> {
        TypeInfoAtPath::Array(self)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ListInfo<'a> {
    node: &'a ListInfoNode,
    graph: &'a TypeInfoGraph,
}

impl<'a> ListInfo<'a> {
    pub fn type_name(self) -> &'a str {
        &self.node.type_name
    }

    pub fn field_type(self) -> TypeInfo<'a> {
        TypeInfo::new(self.node.field_type_id, self.graph)
    }

    fn into_type_info_at_path(self) -> TypeInfoAtPath<'a> {
        TypeInfoAtPath::List(self)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct MapInfo<'a> {
    node: &'a MapInfoNode,
    graph: &'a TypeInfoGraph,
}

impl<'a> MapInfo<'a> {
    pub fn type_name(self) -> &'a str {
        &self.node.type_name
    }

    pub fn key_type(self) -> TypeInfo<'a> {
        TypeInfo::new(self.node.key_type_id, self.graph)
    }

    pub fn value_type(self) -> TypeInfo<'a> {
        TypeInfo::new(self.node.value_type_id, self.graph)
    }

    fn into_type_info_at_path(self) -> TypeInfoAtPath<'a> {
        TypeInfoAtPath::Map(self)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct OpaqueInfo<'a> {
    node: &'a OpaqueInfoNode,
    #[allow(dead_code)]
    graph: &'a TypeInfoGraph,
}

impl<'a> OpaqueInfo<'a> {
    pub fn type_name(self) -> &'a str {
        &self.node.type_name
    }

    fn into_type_info_at_path(self) -> TypeInfoAtPath<'a> {
        TypeInfoAtPath::Opaque(self)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum TypeInfoAtPath<'a> {
    Struct(StructInfo<'a>),
    TupleStruct(TupleStructInfo<'a>),
    Tuple(TupleInfo<'a>),
    Enum(EnumInfo<'a>),
    Variant(VariantInfo<'a>),
    List(ListInfo<'a>),
    Array(ArrayInfo<'a>),
    Map(MapInfo<'a>),
    Scalar(ScalarInfo),
    Opaque(OpaqueInfo<'a>),
}

impl<'a> GetMeta<'a> for TypeInfoAtPath<'a> {
    fn meta(self, key: &str) -> Option<&'a dyn Reflect> {
        match self {
            TypeInfoAtPath::Struct(inner) => inner.meta(key),
            TypeInfoAtPath::TupleStruct(inner) => inner.meta(key),
            TypeInfoAtPath::Enum(inner) => inner.meta(key),
            TypeInfoAtPath::Opaque(inner) => inner.meta(key),
            TypeInfoAtPath::Variant(_)
            | TypeInfoAtPath::Tuple(_)
            | TypeInfoAtPath::List(_)
            | TypeInfoAtPath::Array(_)
            | TypeInfoAtPath::Map(_)
            | TypeInfoAtPath::Scalar(_) => None,
        }
    }

    fn docs(self) -> &'a [String] {
        match self {
            TypeInfoAtPath::Struct(inner) => inner.docs(),
            TypeInfoAtPath::TupleStruct(inner) => inner.docs(),
            TypeInfoAtPath::Enum(inner) => inner.docs(),
            TypeInfoAtPath::Variant(_)
            | TypeInfoAtPath::Tuple(_)
            | TypeInfoAtPath::List(_)
            | TypeInfoAtPath::Array(_)
            | TypeInfoAtPath::Map(_)
            | TypeInfoAtPath::Scalar(_)
            | TypeInfoAtPath::Opaque(_) => &[],
        }
    }
}

impl<'a> TypeInfoAtPath<'a> {
    pub fn as_struct(self) -> Option<StructInfo<'a>> {
        match self {
            Self::Struct(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_tuple_struct(self) -> Option<TupleStructInfo<'a>> {
        match self {
            Self::TupleStruct(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_tuple(self) -> Option<TupleInfo<'a>> {
        match self {
            Self::Tuple(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_enum(self) -> Option<EnumInfo<'a>> {
        match self {
            Self::Enum(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_variant(self) -> Option<VariantInfo<'a>> {
        match self {
            Self::Variant(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_array(self) -> Option<ArrayInfo<'a>> {
        match self {
            Self::Array(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_list(self) -> Option<ListInfo<'a>> {
        match self {
            Self::List(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_map(self) -> Option<MapInfo<'a>> {
        match self {
            Self::Map(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_scalar(self) -> Option<ScalarInfo> {
        match self {
            Self::Scalar(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_opaque(self) -> Option<OpaqueInfo<'a>> {
        match self {
            Self::Opaque(inner) => Some(inner),
            _ => None,
        }
    }
}

impl<'a> GetTypedPath<'a> for TypeInfoAtPath<'a> {
    fn at_typed(self, key_path: KeyPath) -> Option<TypeInfoAtPath<'a>> {
        fn go(type_info: TypeInfoAtPath<'_>, mut stack: Vec<Key>) -> Option<TypeInfoAtPath<'_>> {
            let head = stack.pop()?;

            let value_at_key: TypeInfoAtPath<'_> = match head {
                Key::Field(key) => match type_info {
                    TypeInfoAtPath::Struct(struct_) => struct_
                        .fields()
                        .find(|field| field.name() == key)?
                        .type_()
                        .into_type_info_at_path(),
                    TypeInfoAtPath::Map(map) => map.value_type().into_type_info_at_path(),
                    TypeInfoAtPath::Variant(variant) => match variant {
                        VariantInfo::Struct(struct_variant) => struct_variant
                            .fields()
                            .find(|field| field.name() == key)?
                            .type_()
                            .into_type_info_at_path(),
                        VariantInfo::Tuple(_) | VariantInfo::Unit(_) => return None,
                    },
                    TypeInfoAtPath::Enum(enum_) => {
                        let mut variants = enum_.variants();
                        let first = variants.next()?;
                        if variants.next().is_none() {
                            first
                                .fields()
                                .find_map(|field| match field {
                                    VariantField::Named(field) => {
                                        if field.name() == key {
                                            Some(field)
                                        } else {
                                            None
                                        }
                                    }
                                    VariantField::Unnamed(_) => None,
                                })?
                                .type_()
                                .into_type_info_at_path()
                        } else {
                            return None;
                        }
                    }
                    TypeInfoAtPath::TupleStruct(_)
                    | TypeInfoAtPath::Tuple(_)
                    | TypeInfoAtPath::List(_)
                    | TypeInfoAtPath::Array(_)
                    | TypeInfoAtPath::Scalar(_)
                    | TypeInfoAtPath::Opaque(_) => return None,
                },
                Key::Element(index) => match type_info {
                    TypeInfoAtPath::List(list) => list.field_type().into_type_info_at_path(),
                    TypeInfoAtPath::Array(array) => array.field_type().into_type_info_at_path(),
                    TypeInfoAtPath::Map(map) => map.value_type().into_type_info_at_path(),
                    TypeInfoAtPath::TupleStruct(tuple_struct) => tuple_struct
                        .fields()
                        .nth(index)?
                        .type_()
                        .into_type_info_at_path(),
                    TypeInfoAtPath::Tuple(tuple) => {
                        tuple.fields().nth(index)?.type_().into_type_info_at_path()
                    }

                    TypeInfoAtPath::Variant(variant) => match variant {
                        VariantInfo::Tuple(tuple_variant) => tuple_variant
                            .fields()
                            .nth(index)?
                            .type_()
                            .into_type_info_at_path(),
                        VariantInfo::Struct(_) | VariantInfo::Unit(_) => return None,
                    },

                    TypeInfoAtPath::Enum(_)
                    | TypeInfoAtPath::Scalar(_)
                    | TypeInfoAtPath::Struct(_)
                    | TypeInfoAtPath::Opaque(_) => return None,
                },
                Key::Variant(variant) => match type_info {
                    TypeInfoAtPath::Variant(v) => {
                        if v.name() == variant {
                            TypeInfoAtPath::Variant(v)
                        } else {
                            return None;
                        }
                    }
                    TypeInfoAtPath::Enum(enum_) => {
                        let variant_info = enum_.variants().find(|v| v.name() == variant)?;
                        TypeInfoAtPath::Variant(variant_info)
                    }
                    TypeInfoAtPath::Struct(_)
                    | TypeInfoAtPath::TupleStruct(_)
                    | TypeInfoAtPath::Tuple(_)
                    | TypeInfoAtPath::List(_)
                    | TypeInfoAtPath::Array(_)
                    | TypeInfoAtPath::Map(_)
                    | TypeInfoAtPath::Scalar(_)
                    | TypeInfoAtPath::Opaque(_) => return None,
                },
            };

            if stack.is_empty() {
                Some(value_at_key)
            } else {
                go(value_at_key, stack)
            }
        }

        if key_path.is_empty() {
            return Some(self);
        }

        let mut path = key_path.path;
        path.reverse();
        go(self, path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Reflect;

    #[test]
    fn struct_() {
        #[derive(Reflect, Clone, Debug)]
        #[reflect(crate_name(crate))]
        struct Foo {
            n: i32,
            foos: Vec<Foo>,
        }

        let type_info = <Foo as Typed>::type_info();

        assert_eq!(
            type_info.type_().type_name(),
            "mirror_mirror::type_info::tests::struct_::Foo"
        );

        let struct_ = type_info.type_().as_struct().unwrap();

        assert_eq!(
            struct_.type_name(),
            "mirror_mirror::type_info::tests::struct_::Foo"
        );

        for field in struct_.fields() {
            match field.name() {
                "foos" => {
                    assert_eq!(
                        field.type_().type_name(),
                        "alloc::vec::Vec<mirror_mirror::type_info::tests::struct_::Foo>"
                    );

                    let list = field.type_().as_list().unwrap();

                    assert_eq!(
                        list.type_name(),
                        "alloc::vec::Vec<mirror_mirror::type_info::tests::struct_::Foo>"
                    );

                    assert_eq!(
                        list.field_type().type_name(),
                        "mirror_mirror::type_info::tests::struct_::Foo"
                    );
                }
                "n" => {
                    assert_eq!(field.type_().type_name(), "i32");
                    let scalar = field.type_().as_scalar().unwrap();
                    assert_eq!(scalar.type_name(), "i32");
                }
                _ => panic!("wat"),
            }
        }
    }

    #[test]
    fn enum_() {
        #[derive(Reflect, Clone, Debug)]
        #[reflect(crate_name(crate))]
        enum Foo {
            A { a: String },
            B(Vec<Foo>),
            C,
        }
    }
}
