use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use graph::*;

use crate::enum_::EnumValue;
use crate::key_path::GetTypePath;
use crate::key_path::Key;
use crate::key_path::KeyPath;
use crate::struct_::StructValue;
use crate::tuple::TupleValue;
use crate::tuple_struct::TupleStructValue;
use crate::Reflect;
use crate::Value;

pub mod graph;

pub trait Typed: 'static {
    fn type_info() -> TypeRoot {
        let mut graph = TypeGraph::default();
        let id = Self::build(&mut graph);
        TypeRoot { root: id, graph }
    }

    fn build(graph: &mut TypeGraph) -> NodeId;
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "speedy", derive(speedy::Readable, speedy::Writable))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TypeRoot {
    root: NodeId,
    graph: TypeGraph,
}

impl TypeRoot {
    pub fn get_type(&self) -> Type<'_> {
        Type::new(self.root, &self.graph)
    }

    pub fn type_name(&self) -> &str {
        self.get_type().type_name()
    }

    pub fn default_value(&self) -> Option<Value> {
        self.get_type().default_value()
    }

    pub fn as_struct(&self) -> Option<StructType<'_>> {
        self.get_type().as_struct()
    }

    pub fn as_tuple_struct(&self) -> Option<TupleStructType<'_>> {
        self.get_type().as_tuple_struct()
    }

    pub fn as_tuple(&self) -> Option<TupleType<'_>> {
        self.get_type().as_tuple()
    }

    pub fn as_enum(&self) -> Option<EnumType<'_>> {
        self.get_type().as_enum()
    }

    pub fn as_array(&self) -> Option<ArrayType<'_>> {
        self.get_type().as_array()
    }

    pub fn as_list(&self) -> Option<ListType<'_>> {
        self.get_type().as_list()
    }

    pub fn as_map(&self) -> Option<MapType<'_>> {
        self.get_type().as_map()
    }

    pub fn as_scalar(&self) -> Option<ScalarType> {
        self.get_type().as_scalar()
    }

    pub fn as_opaque(&self) -> Option<OpaqueType<'_>> {
        self.get_type().as_opaque()
    }
}

impl<'a> GetTypePath<'a> for &'a TypeRoot {
    fn at_type(self, key_path: KeyPath) -> Option<TypeAtPath<'a>> {
        self.get_type().at_type(key_path)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Type<'a> {
    Struct(StructType<'a>),
    TupleStruct(TupleStructType<'a>),
    Tuple(TupleType<'a>),
    Enum(EnumType<'a>),
    List(ListType<'a>),
    Array(ArrayType<'a>),
    Map(MapType<'a>),
    Scalar(ScalarType),
    Opaque(OpaqueType<'a>),
}

impl<'a> Type<'a> {
    fn new(id: NodeId, graph: &'a TypeGraph) -> Self {
        match graph.get(id) {
            TypeNode::Struct(node) => {
                let node = StructType { node, graph };
                Type::Struct(node)
            }
            TypeNode::TupleStruct(node) => {
                let node = TupleStructType { node, graph };
                Type::TupleStruct(node)
            }
            TypeNode::Tuple(node) => {
                let node = TupleType { node, graph };
                Type::Tuple(node)
            }
            TypeNode::Enum(node) => {
                let node = EnumType { node, graph };
                Type::Enum(node)
            }
            TypeNode::List(node) => {
                let node = ListType { node, graph };
                Type::List(node)
            }
            TypeNode::Array(node) => {
                let node = ArrayType { node, graph };
                Type::Array(node)
            }
            TypeNode::Map(node) => {
                let node = MapType { node, graph };
                Type::Map(node)
            }
            TypeNode::Scalar(scalar) => {
                let node = match scalar {
                    ScalarNode::usize => ScalarType::usize,
                    ScalarNode::u8 => ScalarType::u8,
                    ScalarNode::u16 => ScalarType::u16,
                    ScalarNode::u32 => ScalarType::u32,
                    ScalarNode::u64 => ScalarType::u64,
                    ScalarNode::u128 => ScalarType::u128,
                    ScalarNode::i8 => ScalarType::i8,
                    ScalarNode::i16 => ScalarType::i16,
                    ScalarNode::i32 => ScalarType::i32,
                    ScalarNode::i64 => ScalarType::i64,
                    ScalarNode::i128 => ScalarType::i128,
                    ScalarNode::bool => ScalarType::bool,
                    ScalarNode::char => ScalarType::char,
                    ScalarNode::f32 => ScalarType::f32,
                    ScalarNode::f64 => ScalarType::f64,
                    ScalarNode::String => ScalarType::String,
                };
                Type::Scalar(node)
            }
            TypeNode::Opaque(node) => {
                let node = OpaqueType { node, graph };
                Type::Opaque(node)
            }
        }
    }

    pub fn type_name(self) -> &'a str {
        match self {
            Type::Struct(inner) => inner.type_name(),
            Type::TupleStruct(inner) => inner.type_name(),
            Type::Tuple(inner) => inner.type_name(),
            Type::Enum(inner) => inner.type_name(),
            Type::List(inner) => inner.type_name(),
            Type::Array(inner) => inner.type_name(),
            Type::Map(inner) => inner.type_name(),
            Type::Scalar(inner) => inner.type_name(),
            Type::Opaque(inner) => inner.type_name(),
        }
    }

    fn into_type_info_at_path(self) -> TypeAtPath<'a> {
        match self {
            Type::Struct(inner) => inner.into_type_info_at_path(),
            Type::TupleStruct(inner) => inner.into_type_info_at_path(),
            Type::Tuple(inner) => inner.into_type_info_at_path(),
            Type::Enum(inner) => inner.into_type_info_at_path(),
            Type::List(inner) => inner.into_type_info_at_path(),
            Type::Array(inner) => inner.into_type_info_at_path(),
            Type::Map(inner) => inner.into_type_info_at_path(),
            Type::Scalar(inner) => inner.into_type_info_at_path(),
            Type::Opaque(inner) => inner.into_type_info_at_path(),
        }
    }

    pub fn default_value(self) -> Option<Value> {
        let value = match self {
            Type::Struct(struct_) => {
                let mut value = StructValue::new();
                for field in struct_.field_types() {
                    value.set_field(field.name(), field.get_type().default_value()?);
                }
                value.to_value()
            }
            Type::TupleStruct(tuple_struct) => {
                let mut value = TupleStructValue::new();
                for field in tuple_struct.field_types() {
                    value.push_field(field.get_type().default_value()?);
                }
                value.to_value()
            }
            Type::Tuple(tuple) => {
                let mut value = TupleValue::new();
                for field in tuple.field_types() {
                    value.push_field(field.get_type().default_value()?);
                }
                value.to_value()
            }
            Type::Enum(enum_) => {
                let mut variants = enum_.variants();
                let variant = variants.next()?;
                match variant {
                    Variant::Struct(variant) => {
                        let mut value = EnumValue::new_struct_variant(variant.name());
                        for field in variant.field_types() {
                            value.set_struct_field(field.name(), field.get_type().default_value()?);
                        }
                        value.to_value()
                    }
                    Variant::Tuple(variant) => {
                        let mut value = EnumValue::new_tuple_variant(variant.name());
                        for field in variant.field_types() {
                            value.push_tuple_field(field.get_type().default_value()?);
                        }
                        value.to_value()
                    }
                    Variant::Unit(variant) => {
                        EnumValue::new_unit_variant(variant.name()).to_value()
                    }
                }
            }
            Type::List(_) => Vec::<()>::new().to_value(),
            Type::Array(_) => <[(); 0] as Reflect>::to_value(&[]),
            Type::Map(_) => BTreeMap::<(), ()>::new().to_value(),
            Type::Scalar(scalar) => match scalar {
                ScalarType::usize => usize::default().to_value(),
                ScalarType::u8 => u8::default().to_value(),
                ScalarType::u16 => u16::default().to_value(),
                ScalarType::u32 => u32::default().to_value(),
                ScalarType::u64 => u64::default().to_value(),
                ScalarType::u128 => u128::default().to_value(),
                ScalarType::i8 => i8::default().to_value(),
                ScalarType::i16 => i16::default().to_value(),
                ScalarType::i32 => i32::default().to_value(),
                ScalarType::i64 => i64::default().to_value(),
                ScalarType::i128 => i128::default().to_value(),
                ScalarType::bool => bool::default().to_value(),
                ScalarType::char => char::default().to_value(),
                ScalarType::f32 => f32::default().to_value(),
                ScalarType::f64 => f64::default().to_value(),
                ScalarType::String => String::default().to_value(),
            },
            Type::Opaque(_) => return None,
        };
        Some(value)
    }

    pub fn as_struct(self) -> Option<StructType<'a>> {
        match self {
            Type::Struct(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_tuple_struct(self) -> Option<TupleStructType<'a>> {
        match self {
            Type::TupleStruct(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_tuple(self) -> Option<TupleType<'a>> {
        match self {
            Type::Tuple(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_enum(self) -> Option<EnumType<'a>> {
        match self {
            Type::Enum(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_array(self) -> Option<ArrayType<'a>> {
        match self {
            Type::Array(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_list(self) -> Option<ListType<'a>> {
        match self {
            Type::List(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_map(self) -> Option<MapType<'a>> {
        match self {
            Type::Map(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_scalar(self) -> Option<ScalarType> {
        match self {
            Type::Scalar(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_opaque(self) -> Option<OpaqueType<'a>> {
        match self {
            Type::Opaque(inner) => Some(inner),
            _ => None,
        }
    }
}

impl<'a> GetTypePath<'a> for Type<'a> {
    fn at_type(self, key_path: KeyPath) -> Option<TypeAtPath<'a>> {
        self.into_type_info_at_path().at_type(key_path)
    }
}

impl<'a> GetTypePath<'a> for StructType<'a> {
    fn at_type(self, key_path: KeyPath) -> Option<TypeAtPath<'a>> {
        self.into_type_info_at_path().at_type(key_path)
    }
}

impl<'a> GetTypePath<'a> for TupleStructType<'a> {
    fn at_type(self, key_path: KeyPath) -> Option<TypeAtPath<'a>> {
        self.into_type_info_at_path().at_type(key_path)
    }
}

impl<'a> GetTypePath<'a> for TupleType<'a> {
    fn at_type(self, key_path: KeyPath) -> Option<TypeAtPath<'a>> {
        self.into_type_info_at_path().at_type(key_path)
    }
}

impl<'a> GetTypePath<'a> for EnumType<'a> {
    fn at_type(self, key_path: KeyPath) -> Option<TypeAtPath<'a>> {
        self.into_type_info_at_path().at_type(key_path)
    }
}

impl<'a> GetTypePath<'a> for ListType<'a> {
    fn at_type(self, key_path: KeyPath) -> Option<TypeAtPath<'a>> {
        self.into_type_info_at_path().at_type(key_path)
    }
}

impl<'a> GetTypePath<'a> for MapType<'a> {
    fn at_type(self, key_path: KeyPath) -> Option<TypeAtPath<'a>> {
        self.into_type_info_at_path().at_type(key_path)
    }
}

impl GetTypePath<'static> for ScalarType {
    fn at_type(self, key_path: KeyPath) -> Option<TypeAtPath<'static>> {
        self.into_type_info_at_path().at_type(key_path)
    }
}

impl<'a> GetTypePath<'a> for Variant<'a> {
    fn at_type(self, key_path: KeyPath) -> Option<TypeAtPath<'a>> {
        self.into_type_info_at_path().at_type(key_path)
    }
}

mod private {
    use super::*;

    pub trait Sealed {}

    impl Sealed for Type<'_> {}
    impl Sealed for StructType<'_> {}
    impl Sealed for TupleStructType<'_> {}
    impl Sealed for EnumType<'_> {}
    impl Sealed for StructVariant<'_> {}
    impl Sealed for TupleVariant<'_> {}
    impl Sealed for UnitVariant<'_> {}
    impl Sealed for UnnamedField<'_> {}
    impl Sealed for NamedField<'_> {}
    impl Sealed for OpaqueType<'_> {}
    impl Sealed for Variant<'_> {}
    impl Sealed for VariantField<'_> {}
    impl Sealed for TypeAtPath<'_> {}
}

pub trait GetMeta<'a>: private::Sealed {
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

impl<'a> GetMeta<'a> for Type<'a> {
    fn meta(self, key: &str) -> Option<&'a dyn Reflect> {
        match self {
            Type::Struct(inner) => inner.meta(key),
            Type::TupleStruct(inner) => inner.meta(key),
            Type::Enum(inner) => inner.meta(key),
            Type::Opaque(inner) => inner.meta(key),
            Type::Tuple(_)
            | Type::List(_)
            | Type::Array(_)
            | Type::Map(_)
            | Type::Scalar(_) => None,
        }
    }

    fn docs(self) -> &'a [String] {
        match self {
            Type::Struct(inner) => inner.docs(),
            Type::TupleStruct(inner) => inner.docs(),
            Type::Enum(inner) => inner.docs(),
            Type::Tuple(_)
            | Type::List(_)
            | Type::Array(_)
            | Type::Map(_)
            | Type::Scalar(_)
            | Type::Opaque(_) => &[],
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
    StructType
    TupleStructType
    EnumType
    StructVariant
    TupleVariant
    UnitVariant
    UnnamedField
    NamedField
}

impl<'a> GetMeta<'a> for OpaqueType<'a> {
    fn meta(self, key: &str) -> Option<&'a dyn Reflect> {
        Some(self.node.metadata.get(key)?.as_reflect())
    }

    fn docs(self) -> &'a [String] {
        &[]
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "speedy", derive(speedy::Readable, speedy::Writable))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ScalarType {
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

impl ScalarType {
    pub fn type_name(self) -> &'static str {
        match self {
            ScalarType::usize => "usize",
            ScalarType::u8 => "u8",
            ScalarType::u16 => "u16",
            ScalarType::u32 => "u32",
            ScalarType::u64 => "u64",
            ScalarType::u128 => "u128",
            ScalarType::i8 => "i8",
            ScalarType::i16 => "i16",
            ScalarType::i32 => "i32",
            ScalarType::i64 => "i64",
            ScalarType::i128 => "i128",
            ScalarType::bool => "bool",
            ScalarType::char => "char",
            ScalarType::f32 => "f32",
            ScalarType::f64 => "f64",
            ScalarType::String => "String",
        }
    }

    fn into_type_info_at_path(self) -> TypeAtPath<'static> {
        TypeAtPath::Scalar(self)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct StructType<'a> {
    node: &'a StructNode,
    graph: &'a TypeGraph,
}

impl<'a> StructType<'a> {
    pub fn type_name(self) -> &'a str {
        &self.node.type_name
    }

    pub fn field_types(self) -> impl Iterator<Item = NamedField<'a>> {
        self.node.fields.values().map(|node| NamedField {
            node,
            graph: self.graph,
        })
    }

    pub fn field_type(self, name: &str) -> Option<NamedField<'a>> {
        let node = self.node.fields.get(name)?;
        Some(NamedField {
            node,
            graph: self.graph,
        })
    }

    pub fn field_type_at(self, index: usize) -> Option<NamedField<'a>> {
        let name = self.node.field_names.get(index)?;
        self.field_type(name)
    }

    fn into_type_info_at_path(self) -> TypeAtPath<'a> {
        TypeAtPath::Struct(self)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct TupleStructType<'a> {
    node: &'a TupleStructNode,
    graph: &'a TypeGraph,
}

impl<'a> TupleStructType<'a> {
    pub fn type_name(self) -> &'a str {
        &self.node.type_name
    }

    pub fn field_types(self) -> impl Iterator<Item = UnnamedField<'a>> {
        self.node.fields.iter().map(|node| UnnamedField {
            node,
            graph: self.graph,
        })
    }

    pub fn field_type(self, index: usize) -> Option<UnnamedField<'a>> {
        let node = self.node.fields.get(index)?;
        Some(UnnamedField {
            node,
            graph: self.graph,
        })
    }

    fn into_type_info_at_path(self) -> TypeAtPath<'a> {
        TypeAtPath::TupleStruct(self)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct TupleType<'a> {
    node: &'a TupleNode,
    graph: &'a TypeGraph,
}

impl<'a> TupleType<'a> {
    pub fn type_name(self) -> &'a str {
        &self.node.type_name
    }

    pub fn field_types(self) -> impl Iterator<Item = UnnamedField<'a>> {
        self.node.fields.iter().map(|node| UnnamedField {
            node,
            graph: self.graph,
        })
    }

    pub fn field_type(self, index: usize) -> Option<UnnamedField<'a>> {
        let node = self.node.fields.get(index)?;
        Some(UnnamedField {
            node,
            graph: self.graph,
        })
    }

    fn into_type_info_at_path(self) -> TypeAtPath<'a> {
        TypeAtPath::Tuple(self)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct EnumType<'a> {
    node: &'a EnumNode,
    graph: &'a TypeGraph,
}

impl<'a> EnumType<'a> {
    pub fn type_name(self) -> &'a str {
        &self.node.type_name
    }

    pub fn variants(self) -> impl Iterator<Item = Variant<'a>> {
        self.node.variants.iter().map(|variant| match variant {
            VariantNode::Struct(node) => Variant::Struct(StructVariant {
                node,
                enum_node: self.node,
                graph: self.graph,
            }),
            VariantNode::Tuple(node) => Variant::Tuple(TupleVariant {
                node,
                enum_node: self.node,
                graph: self.graph,
            }),
            VariantNode::Unit(node) => Variant::Unit(UnitVariant {
                node,
                enum_node: self.node,
                graph: self.graph,
            }),
        })
    }

    fn into_type_info_at_path(self) -> TypeAtPath<'a> {
        TypeAtPath::Enum(self)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Variant<'a> {
    Struct(StructVariant<'a>),
    Tuple(TupleVariant<'a>),
    Unit(UnitVariant<'a>),
}

impl<'a> Variant<'a> {
    pub fn name(self) -> &'a str {
        match self {
            Variant::Struct(inner) => inner.name(),
            Variant::Tuple(inner) => inner.name(),
            Variant::Unit(inner) => inner.name(),
        }
    }

    pub fn field_types(self) -> impl Iterator<Item = VariantField<'a>> {
        match self {
            Variant::Struct(inner) => Box::new(inner.field_types().map(VariantField::Named))
                as Box<dyn Iterator<Item = VariantField<'a>>>,
            Variant::Tuple(inner) => Box::new(inner.field_types().map(VariantField::Unnamed)),
            Variant::Unit(_) => Box::new(core::iter::empty()),
        }
    }

    pub fn field_type(self, name: &str) -> Option<NamedField<'a>> {
        match self {
            Variant::Struct(inner) => inner.field_type(name),
            Variant::Tuple(_) | Variant::Unit(_) => None,
        }
    }

    pub fn field_type_at(self, index: usize) -> Option<VariantField<'a>> {
        match self {
            Variant::Struct(inner) => inner.field_type_at(index).map(VariantField::Named),
            Variant::Tuple(inner) => inner.field_type(index).map(VariantField::Unnamed),
            Variant::Unit(_) => None,
        }
    }

    pub fn enum_type(self) -> EnumType<'a> {
        match self {
            Variant::Struct(inner) => inner.enum_type(),
            Variant::Tuple(inner) => inner.enum_type(),
            Variant::Unit(inner) => inner.enum_type(),
        }
    }

    fn into_type_info_at_path(self) -> TypeAtPath<'a> {
        TypeAtPath::Variant(self)
    }
}

impl<'a> GetMeta<'a> for Variant<'a> {
    fn meta(self, key: &str) -> Option<&'a dyn Reflect> {
        match self {
            Variant::Struct(inner) => inner.meta(key),
            Variant::Tuple(inner) => inner.meta(key),
            Variant::Unit(inner) => inner.meta(key),
        }
    }

    fn docs(self) -> &'a [String] {
        match self {
            Variant::Struct(inner) => inner.docs(),
            Variant::Tuple(inner) => inner.docs(),
            Variant::Unit(inner) => inner.docs(),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum VariantField<'a> {
    Named(NamedField<'a>),
    Unnamed(UnnamedField<'a>),
}

impl<'a> VariantField<'a> {
    pub fn get_type(self) -> Type<'a> {
        match self {
            VariantField::Named(inner) => inner.get_type(),
            VariantField::Unnamed(inner) => inner.get_type(),
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
pub struct StructVariant<'a> {
    node: &'a StructVariantNode,
    enum_node: &'a EnumNode,
    graph: &'a TypeGraph,
}

impl<'a> StructVariant<'a> {
    pub fn name(self) -> &'a str {
        &self.node.name
    }

    pub fn field_types(self) -> impl Iterator<Item = NamedField<'a>> {
        self.node.fields.values().map(|node| NamedField {
            node,
            graph: self.graph,
        })
    }

    pub fn field_type(self, name: &str) -> Option<NamedField<'a>> {
        let node = self.node.fields.get(name)?;
        Some(NamedField {
            node,
            graph: self.graph,
        })
    }

    pub fn field_type_at(self, index: usize) -> Option<NamedField<'a>> {
        let name = self.node.field_names.get(index)?;
        self.field_type(name)
    }

    pub fn enum_type(self) -> EnumType<'a> {
        EnumType {
            node: self.enum_node,
            graph: self.graph,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct TupleVariant<'a> {
    node: &'a TupleVariantNode,
    enum_node: &'a EnumNode,
    graph: &'a TypeGraph,
}

impl<'a> TupleVariant<'a> {
    pub fn name(self) -> &'a str {
        &self.node.name
    }

    pub fn field_types(self) -> impl Iterator<Item = UnnamedField<'a>> {
        self.node.fields.iter().map(|node| UnnamedField {
            node,
            graph: self.graph,
        })
    }

    pub fn field_type(self, index: usize) -> Option<UnnamedField<'a>> {
        let node = self.node.fields.get(index)?;
        Some(UnnamedField {
            node,
            graph: self.graph,
        })
    }

    pub fn enum_type(self) -> EnumType<'a> {
        EnumType {
            node: self.enum_node,
            graph: self.graph,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct UnitVariant<'a> {
    node: &'a UnitVariantNode,
    enum_node: &'a EnumNode,
    graph: &'a TypeGraph,
}

impl<'a> UnitVariant<'a> {
    pub fn name(self) -> &'a str {
        &self.node.name
    }

    pub fn enum_type(self) -> EnumType<'a> {
        EnumType {
            node: self.enum_node,
            graph: self.graph,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct UnnamedField<'a> {
    node: &'a UnnamedFieldNode,
    graph: &'a TypeGraph,
}

impl<'a> UnnamedField<'a> {
    pub fn get_type(self) -> Type<'a> {
        Type::new(self.node.id, self.graph)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct NamedField<'a> {
    node: &'a NamedFieldNode,
    graph: &'a TypeGraph,
}

impl<'a> NamedField<'a> {
    pub fn name(self) -> &'a str {
        &self.node.name
    }

    pub fn get_type(self) -> Type<'a> {
        Type::new(self.node.id, self.graph)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ArrayType<'a> {
    node: &'a ArrayNode,
    graph: &'a TypeGraph,
}

impl<'a> ArrayType<'a> {
    pub fn type_name(self) -> &'a str {
        &self.node.type_name
    }

    pub fn field_type(self) -> Type<'a> {
        Type::new(self.node.field_type_id, self.graph)
    }

    pub fn len(self) -> usize {
        self.node.len
    }

    pub fn is_empty(self) -> bool {
        self.node.len == 0
    }

    fn into_type_info_at_path(self) -> TypeAtPath<'a> {
        TypeAtPath::Array(self)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ListType<'a> {
    node: &'a ListNode,
    graph: &'a TypeGraph,
}

impl<'a> ListType<'a> {
    pub fn type_name(self) -> &'a str {
        &self.node.type_name
    }

    pub fn field_type(self) -> Type<'a> {
        Type::new(self.node.field_type_id, self.graph)
    }

    fn into_type_info_at_path(self) -> TypeAtPath<'a> {
        TypeAtPath::List(self)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct MapType<'a> {
    node: &'a MapNode,
    graph: &'a TypeGraph,
}

impl<'a> MapType<'a> {
    pub fn type_name(self) -> &'a str {
        &self.node.type_name
    }

    pub fn key_type(self) -> Type<'a> {
        Type::new(self.node.key_type_id, self.graph)
    }

    pub fn value_type(self) -> Type<'a> {
        Type::new(self.node.value_type_id, self.graph)
    }

    fn into_type_info_at_path(self) -> TypeAtPath<'a> {
        TypeAtPath::Map(self)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct OpaqueType<'a> {
    node: &'a OpaqueNode,
    #[allow(dead_code)]
    graph: &'a TypeGraph,
}

impl<'a> OpaqueType<'a> {
    pub fn type_name(self) -> &'a str {
        &self.node.type_name
    }

    fn into_type_info_at_path(self) -> TypeAtPath<'a> {
        TypeAtPath::Opaque(self)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum TypeAtPath<'a> {
    Struct(StructType<'a>),
    TupleStruct(TupleStructType<'a>),
    Tuple(TupleType<'a>),
    Enum(EnumType<'a>),
    Variant(Variant<'a>),
    List(ListType<'a>),
    Array(ArrayType<'a>),
    Map(MapType<'a>),
    Scalar(ScalarType),
    Opaque(OpaqueType<'a>),
}

impl<'a> GetMeta<'a> for TypeAtPath<'a> {
    fn meta(self, key: &str) -> Option<&'a dyn Reflect> {
        match self {
            TypeAtPath::Struct(inner) => inner.meta(key),
            TypeAtPath::TupleStruct(inner) => inner.meta(key),
            TypeAtPath::Enum(inner) => inner.meta(key),
            TypeAtPath::Opaque(inner) => inner.meta(key),
            TypeAtPath::Variant(_)
            | TypeAtPath::Tuple(_)
            | TypeAtPath::List(_)
            | TypeAtPath::Array(_)
            | TypeAtPath::Map(_)
            | TypeAtPath::Scalar(_) => None,
        }
    }

    fn docs(self) -> &'a [String] {
        match self {
            TypeAtPath::Struct(inner) => inner.docs(),
            TypeAtPath::TupleStruct(inner) => inner.docs(),
            TypeAtPath::Enum(inner) => inner.docs(),
            TypeAtPath::Variant(_)
            | TypeAtPath::Tuple(_)
            | TypeAtPath::List(_)
            | TypeAtPath::Array(_)
            | TypeAtPath::Map(_)
            | TypeAtPath::Scalar(_)
            | TypeAtPath::Opaque(_) => &[],
        }
    }
}

impl<'a> TypeAtPath<'a> {
    pub fn as_struct(self) -> Option<StructType<'a>> {
        match self {
            Self::Struct(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_tuple_struct(self) -> Option<TupleStructType<'a>> {
        match self {
            Self::TupleStruct(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_tuple(self) -> Option<TupleType<'a>> {
        match self {
            Self::Tuple(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_enum(self) -> Option<EnumType<'a>> {
        match self {
            Self::Enum(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_variant(self) -> Option<Variant<'a>> {
        match self {
            Self::Variant(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_array(self) -> Option<ArrayType<'a>> {
        match self {
            Self::Array(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_list(self) -> Option<ListType<'a>> {
        match self {
            Self::List(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_map(self) -> Option<MapType<'a>> {
        match self {
            Self::Map(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_scalar(self) -> Option<ScalarType> {
        match self {
            Self::Scalar(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_opaque(self) -> Option<OpaqueType<'a>> {
        match self {
            Self::Opaque(inner) => Some(inner),
            _ => None,
        }
    }
}

impl<'a> GetTypePath<'a> for TypeAtPath<'a> {
    fn at_type(self, key_path: KeyPath) -> Option<TypeAtPath<'a>> {
        fn go(type_info: TypeAtPath<'_>, mut stack: Vec<Key>) -> Option<TypeAtPath<'_>> {
            let head = stack.pop()?;

            let value_at_key: TypeAtPath<'_> = match head {
                Key::Field(key) => match type_info {
                    TypeAtPath::Struct(struct_) => struct_
                        .field_types()
                        .find(|field| field.name() == key)?
                        .get_type()
                        .into_type_info_at_path(),
                    TypeAtPath::Map(map) => map.value_type().into_type_info_at_path(),
                    TypeAtPath::Variant(variant) => match variant {
                        Variant::Struct(struct_variant) => struct_variant
                            .field_types()
                            .find(|field| field.name() == key)?
                            .get_type()
                            .into_type_info_at_path(),
                        Variant::Tuple(_) | Variant::Unit(_) => return None,
                    },
                    TypeAtPath::Enum(enum_) => {
                        let mut variants = enum_.variants();
                        let first = variants.next()?;
                        if variants.next().is_none() {
                            first
                                .field_types()
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
                                .get_type()
                                .into_type_info_at_path()
                        } else {
                            return None;
                        }
                    }
                    TypeAtPath::TupleStruct(_)
                    | TypeAtPath::Tuple(_)
                    | TypeAtPath::List(_)
                    | TypeAtPath::Array(_)
                    | TypeAtPath::Scalar(_)
                    | TypeAtPath::Opaque(_) => return None,
                },
                Key::Element(index) => match type_info {
                    TypeAtPath::List(list) => list.field_type().into_type_info_at_path(),
                    TypeAtPath::Array(array) => array.field_type().into_type_info_at_path(),
                    TypeAtPath::Map(map) => map.value_type().into_type_info_at_path(),
                    TypeAtPath::TupleStruct(tuple_struct) => tuple_struct
                        .field_types()
                        .nth(index)?
                        .get_type()
                        .into_type_info_at_path(),
                    TypeAtPath::Tuple(tuple) => tuple
                        .field_types()
                        .nth(index)?
                        .get_type()
                        .into_type_info_at_path(),

                    TypeAtPath::Variant(variant) => match variant {
                        Variant::Tuple(tuple_variant) => tuple_variant
                            .field_types()
                            .nth(index)?
                            .get_type()
                            .into_type_info_at_path(),
                        Variant::Struct(_) | Variant::Unit(_) => return None,
                    },

                    TypeAtPath::Enum(_)
                    | TypeAtPath::Scalar(_)
                    | TypeAtPath::Struct(_)
                    | TypeAtPath::Opaque(_) => return None,
                },
                Key::Variant(variant) => match type_info {
                    TypeAtPath::Variant(v) => {
                        if v.name() == variant {
                            TypeAtPath::Variant(v)
                        } else {
                            return None;
                        }
                    }
                    TypeAtPath::Enum(enum_) => {
                        let variant_info = enum_.variants().find(|v| v.name() == variant)?;
                        TypeAtPath::Variant(variant_info)
                    }
                    TypeAtPath::Struct(_)
                    | TypeAtPath::TupleStruct(_)
                    | TypeAtPath::Tuple(_)
                    | TypeAtPath::List(_)
                    | TypeAtPath::Array(_)
                    | TypeAtPath::Map(_)
                    | TypeAtPath::Scalar(_)
                    | TypeAtPath::Opaque(_) => return None,
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
            type_info.get_type().type_name(),
            "mirror_mirror::type_info::tests::struct_::Foo"
        );

        let struct_ = type_info.get_type().as_struct().unwrap();

        assert_eq!(
            struct_.type_name(),
            "mirror_mirror::type_info::tests::struct_::Foo"
        );

        for field in struct_.field_types() {
            match field.name() {
                "foos" => {
                    assert_eq!(
                        field.get_type().type_name(),
                        "alloc::vec::Vec<mirror_mirror::type_info::tests::struct_::Foo>"
                    );

                    let list = field.get_type().as_list().unwrap();

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
                    assert_eq!(field.get_type().type_name(), "i32");
                    let scalar = field.get_type().as_scalar().unwrap();
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
