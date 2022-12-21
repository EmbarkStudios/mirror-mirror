use core::iter::Peekable;

use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use graph::*;

use crate::enum_::EnumValue;
use crate::key_path::value_to_usize;
use crate::key_path::GetTypePath;
use crate::key_path::Key;
use crate::key_path::KeyPath;
use crate::key_path::NamedOrNumbered;
use crate::struct_::StructValue;
use crate::tuple::TupleValue;
use crate::tuple_struct::TupleStructValue;
use crate::FromReflect;
use crate::Reflect;
use crate::Value;

pub mod graph;

/// Trait for accessing type information.
///
/// Will be implemented by `#[derive(Reflect)]`.
pub trait Typed: 'static {
    fn type_descriptor() -> Cow<'static, TypeDescriptor>;

    fn build(graph: &mut TypeGraph) -> NodeId;
}

/// The root of a type.
///
/// Accessed via the [`Typed`] trait.
///
/// `mirror-mirror` represents types as (possibly cyclic) graphs since types can contain
/// themselves. For example `struct Foo(Vec<Foo>)`.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "speedy", derive(speedy::Readable, speedy::Writable))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TypeDescriptor {
    root: NodeId,
    graph: TypeGraph,
}

impl TypeDescriptor {
    #[doc(hidden)]
    pub fn __private_new(root: NodeId, graph: TypeGraph) -> Self {
        Self { root, graph }
    }

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

impl<'a> GetTypePath<'a> for &'a TypeDescriptor {
    fn type_at(self, key_path: &KeyPath) -> Option<TypeAtPath<'a>> {
        self.get_type().type_at(key_path)
    }
}

impl<'a> GetMeta<'a> for &'a TypeDescriptor {
    fn meta(self, key: &str) -> Option<&'a dyn Reflect> {
        self.get_type().meta(key)
    }

    fn docs(self) -> &'a [String] {
        self.get_type().docs()
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
                let node = StructType {
                    node: WithId::new(id, node),
                    graph,
                };
                Type::Struct(node)
            }
            TypeNode::TupleStruct(node) => {
                let node = TupleStructType {
                    node: WithId::new(id, node),
                    graph,
                };
                Type::TupleStruct(node)
            }
            TypeNode::Tuple(node) => {
                let node = TupleType {
                    node: WithId::new(id, node),
                    graph,
                };
                Type::Tuple(node)
            }
            TypeNode::Enum(node) => {
                let node = EnumType {
                    node: WithId::new(id, node),
                    graph,
                };
                Type::Enum(node)
            }
            TypeNode::List(node) => {
                let node = ListType {
                    node: WithId::new(id, node),
                    graph,
                };
                Type::List(node)
            }
            TypeNode::Array(node) => {
                let node = ArrayType {
                    node: WithId::new(id, node),
                    graph,
                };
                Type::Array(node)
            }
            TypeNode::Map(node) => {
                let node = MapType {
                    node: WithId::new(id, node),
                    graph,
                };
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
                let node = OpaqueType {
                    node: WithId::new(id, node),
                    graph,
                };
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
        match self {
            Type::Struct(inner) => inner.default_value(),
            Type::TupleStruct(inner) => inner.default_value(),
            Type::Tuple(inner) => inner.default_value(),
            Type::Enum(inner) => inner.default_value(),
            Type::List(inner) => Some(inner.default_value()),
            Type::Array(inner) => Some(inner.default_value()),
            Type::Map(inner) => Some(inner.default_value()),
            Type::Scalar(inner) => Some(inner.default_value()),
            Type::Opaque(inner) => inner.default_value(),
        }
    }

    pub fn into_type_descriptor(self) -> Cow<'static, TypeDescriptor> {
        match self {
            Type::Struct(inner) => Cow::Owned(inner.into_type_descriptor()),
            Type::TupleStruct(inner) => Cow::Owned(inner.into_type_descriptor()),
            Type::Tuple(inner) => Cow::Owned(inner.into_type_descriptor()),
            Type::Enum(inner) => Cow::Owned(inner.into_type_descriptor()),
            Type::List(inner) => Cow::Owned(inner.into_type_descriptor()),
            Type::Array(inner) => Cow::Owned(inner.into_type_descriptor()),
            Type::Map(inner) => Cow::Owned(inner.into_type_descriptor()),
            Type::Scalar(inner) => match inner {
                ScalarType::usize => <usize as Typed>::type_descriptor(),
                ScalarType::u8 => <u8 as Typed>::type_descriptor(),
                ScalarType::u16 => <u16 as Typed>::type_descriptor(),
                ScalarType::u32 => <u32 as Typed>::type_descriptor(),
                ScalarType::u64 => <u64 as Typed>::type_descriptor(),
                ScalarType::u128 => <u128 as Typed>::type_descriptor(),
                ScalarType::i8 => <i8 as Typed>::type_descriptor(),
                ScalarType::i16 => <i16 as Typed>::type_descriptor(),
                ScalarType::i32 => <i32 as Typed>::type_descriptor(),
                ScalarType::i64 => <i64 as Typed>::type_descriptor(),
                ScalarType::i128 => <i128 as Typed>::type_descriptor(),
                ScalarType::bool => <bool as Typed>::type_descriptor(),
                ScalarType::char => <char as Typed>::type_descriptor(),
                ScalarType::f32 => <f32 as Typed>::type_descriptor(),
                ScalarType::f64 => <f64 as Typed>::type_descriptor(),
                ScalarType::String => <String as Typed>::type_descriptor(),
            },
            Type::Opaque(inner) => Cow::Owned(inner.into_type_descriptor()),
        }
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
    fn type_at(self, key_path: &KeyPath) -> Option<TypeAtPath<'a>> {
        self.into_type_info_at_path().type_at(key_path)
    }
}

impl<'a> GetTypePath<'a> for StructType<'a> {
    fn type_at(self, key_path: &KeyPath) -> Option<TypeAtPath<'a>> {
        self.into_type_info_at_path().type_at(key_path)
    }
}

impl<'a> GetTypePath<'a> for TupleStructType<'a> {
    fn type_at(self, key_path: &KeyPath) -> Option<TypeAtPath<'a>> {
        self.into_type_info_at_path().type_at(key_path)
    }
}

impl<'a> GetTypePath<'a> for TupleType<'a> {
    fn type_at(self, key_path: &KeyPath) -> Option<TypeAtPath<'a>> {
        self.into_type_info_at_path().type_at(key_path)
    }
}

impl<'a> GetTypePath<'a> for EnumType<'a> {
    fn type_at(self, key_path: &KeyPath) -> Option<TypeAtPath<'a>> {
        self.into_type_info_at_path().type_at(key_path)
    }
}

impl<'a> GetTypePath<'a> for ListType<'a> {
    fn type_at(self, key_path: &KeyPath) -> Option<TypeAtPath<'a>> {
        self.into_type_info_at_path().type_at(key_path)
    }
}

impl<'a> GetTypePath<'a> for MapType<'a> {
    fn type_at(self, key_path: &KeyPath) -> Option<TypeAtPath<'a>> {
        self.into_type_info_at_path().type_at(key_path)
    }
}

impl GetTypePath<'static> for ScalarType {
    fn type_at(self, key_path: &KeyPath) -> Option<TypeAtPath<'static>> {
        self.into_type_info_at_path().type_at(key_path)
    }
}

impl<'a> GetTypePath<'a> for Variant<'a> {
    fn type_at(self, key_path: &KeyPath) -> Option<TypeAtPath<'a>> {
        self.into_type_info_at_path().type_at(key_path)
    }
}

mod private {
    use super::*;

    pub trait Sealed {}

    impl<'a> Sealed for &'a TypeDescriptor {}
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

    fn get_meta<T>(self, key: &str) -> Option<T>
    where
        T: FromReflect,
        Self: Sized,
    {
        T::from_reflect(self.meta(key)?)
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
            Type::Tuple(_) | Type::List(_) | Type::Array(_) | Type::Map(_) | Type::Scalar(_) => {
                None
            }
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
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
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

    pub fn default_value(self) -> Value {
        match self {
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
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct StructType<'a> {
    node: WithId<&'a StructNode>,
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

    pub fn into_type_descriptor(self) -> TypeDescriptor {
        TypeDescriptor {
            root: self.node.id,
            graph: self.graph.clone(),
        }
    }

    pub fn default_value(self) -> Option<Value> {
        let mut value = StructValue::new();
        for field in self.field_types() {
            value.set_field(field.name(), field.get_type().default_value()?);
        }
        Some(value.to_value())
    }
}

#[derive(Copy, Clone, Debug)]
pub struct TupleStructType<'a> {
    node: WithId<&'a TupleStructNode>,
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

    pub fn field_type_at(self, index: usize) -> Option<UnnamedField<'a>> {
        let node = self.node.fields.get(index)?;
        Some(UnnamedField {
            node,
            graph: self.graph,
        })
    }

    fn into_type_info_at_path(self) -> TypeAtPath<'a> {
        TypeAtPath::TupleStruct(self)
    }

    pub fn into_type_descriptor(self) -> TypeDescriptor {
        TypeDescriptor {
            root: self.node.id,
            graph: self.graph.clone(),
        }
    }

    pub fn default_value(self) -> Option<Value> {
        let mut value = TupleStructValue::new();
        for field in self.field_types() {
            value.push_field(field.get_type().default_value()?);
        }
        Some(value.to_value())
    }
}

#[derive(Copy, Clone, Debug)]
pub struct TupleType<'a> {
    node: WithId<&'a TupleNode>,
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

    pub fn field_type_at(self, index: usize) -> Option<UnnamedField<'a>> {
        let node = self.node.fields.get(index)?;
        Some(UnnamedField {
            node,
            graph: self.graph,
        })
    }

    fn into_type_info_at_path(self) -> TypeAtPath<'a> {
        TypeAtPath::Tuple(self)
    }

    pub fn into_type_descriptor(self) -> TypeDescriptor {
        TypeDescriptor {
            root: self.node.id,
            graph: self.graph.clone(),
        }
    }

    pub fn default_value(self) -> Option<Value> {
        let mut value = TupleValue::new();
        for field in self.field_types() {
            value.push_field(field.get_type().default_value()?);
        }
        Some(value.to_value())
    }
}

#[derive(Copy, Clone, Debug)]
pub struct EnumType<'a> {
    node: WithId<&'a EnumNode>,
    graph: &'a TypeGraph,
}

impl<'a> EnumType<'a> {
    pub fn type_name(self) -> &'a str {
        &self.node.type_name
    }

    pub fn variants(self) -> impl Iterator<Item = Variant<'a>> {
        self.node.variants.iter().map(move |variant| match variant {
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

    pub fn variant(self, name: &str) -> Option<Variant<'a>> {
        self.variants().find(|variant| variant.name() == name)
    }

    fn into_type_info_at_path(self) -> TypeAtPath<'a> {
        TypeAtPath::Enum(self)
    }

    pub fn into_type_descriptor(self) -> TypeDescriptor {
        TypeDescriptor {
            root: self.node.id,
            graph: self.graph.clone(),
        }
    }

    pub fn default_value(self) -> Option<Value> {
        let mut variants = self.variants();
        let variant = variants.next()?;
        variant.default_value()
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
            Variant::Tuple(inner) => inner.field_type_at(index).map(VariantField::Unnamed),
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

    pub fn default_value(self) -> Option<Value> {
        match self {
            Variant::Struct(variant) => variant.default_value(),
            Variant::Tuple(variant) => variant.default_value(),
            Variant::Unit(variant) => Some(variant.default_value()),
        }
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
    enum_node: WithId<&'a EnumNode>,
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

    pub fn default_value(self) -> Option<Value> {
        let mut value = EnumValue::new_struct_variant(self.name());
        for field in self.field_types() {
            value.set_struct_field(field.name(), field.get_type().default_value()?);
        }
        Some(value.finish().to_value())
    }
}

#[derive(Copy, Clone, Debug)]
pub struct TupleVariant<'a> {
    node: &'a TupleVariantNode,
    enum_node: WithId<&'a EnumNode>,
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

    pub fn field_type_at(self, index: usize) -> Option<UnnamedField<'a>> {
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

    pub fn default_value(self) -> Option<Value> {
        let mut value = EnumValue::new_tuple_variant(self.name());
        for field in self.field_types() {
            value.push_tuple_field(field.get_type().default_value()?);
        }
        Some(value.finish().to_value())
    }
}

#[derive(Copy, Clone, Debug)]
pub struct UnitVariant<'a> {
    node: &'a UnitVariantNode,
    enum_node: WithId<&'a EnumNode>,
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

    pub fn default_value(self) -> Value {
        EnumValue::new_unit_variant(self.name()).to_value()
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

    fn into_type_info_at_path(self) -> TypeAtPath<'a> {
        self.get_type().into_type_info_at_path()
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

    fn into_type_info_at_path(self) -> TypeAtPath<'a> {
        self.get_type().into_type_info_at_path()
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ArrayType<'a> {
    node: WithId<&'a ArrayNode>,
    graph: &'a TypeGraph,
}

impl<'a> ArrayType<'a> {
    pub fn type_name(self) -> &'a str {
        &self.node.type_name
    }

    pub fn element_type(self) -> Type<'a> {
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

    pub fn into_type_descriptor(self) -> TypeDescriptor {
        TypeDescriptor {
            root: self.node.id,
            graph: self.graph.clone(),
        }
    }

    pub fn default_value(self) -> Value {
        <[(); 0] as Reflect>::to_value(&[])
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ListType<'a> {
    node: WithId<&'a ListNode>,
    graph: &'a TypeGraph,
}

impl<'a> ListType<'a> {
    pub fn type_name(self) -> &'a str {
        &self.node.type_name
    }

    pub fn element_type(self) -> Type<'a> {
        Type::new(self.node.field_type_id, self.graph)
    }

    fn into_type_info_at_path(self) -> TypeAtPath<'a> {
        TypeAtPath::List(self)
    }

    pub fn into_type_descriptor(self) -> TypeDescriptor {
        TypeDescriptor {
            root: self.node.id,
            graph: self.graph.clone(),
        }
    }

    pub fn default_value(self) -> Value {
        Vec::<()>::new().to_value()
    }
}

#[derive(Copy, Clone, Debug)]
pub struct MapType<'a> {
    node: WithId<&'a MapNode>,
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

    pub fn into_type_descriptor(self) -> TypeDescriptor {
        TypeDescriptor {
            root: self.node.id,
            graph: self.graph.clone(),
        }
    }

    pub fn default_value(self) -> Value {
        BTreeMap::<(), ()>::new().to_value()
    }
}

#[derive(Copy, Clone, Debug)]
pub struct OpaqueType<'a> {
    node: WithId<&'a OpaqueNode>,
    graph: &'a TypeGraph,
}

impl<'a> OpaqueType<'a> {
    pub fn type_name(self) -> &'a str {
        &self.node.type_name
    }

    fn into_type_info_at_path(self) -> TypeAtPath<'a> {
        TypeAtPath::Opaque(self)
    }

    pub fn into_type_descriptor(self) -> TypeDescriptor {
        TypeDescriptor {
            root: self.node.id,
            graph: self.graph.clone(),
        }
    }

    pub fn default_value(self) -> Option<Value> {
        None
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
    pub fn default_value(self) -> Option<Value> {
        match self {
            TypeAtPath::Struct(inner) => inner.default_value(),
            TypeAtPath::TupleStruct(inner) => inner.default_value(),
            TypeAtPath::Tuple(inner) => inner.default_value(),
            TypeAtPath::Enum(inner) => inner.default_value(),
            TypeAtPath::Variant(inner) => inner.default_value(),
            TypeAtPath::List(inner) => Some(inner.default_value()),
            TypeAtPath::Array(inner) => Some(inner.default_value()),
            TypeAtPath::Map(inner) => Some(inner.default_value()),
            TypeAtPath::Scalar(inner) => Some(inner.default_value()),
            TypeAtPath::Opaque(inner) => inner.default_value(),
        }
    }

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
    fn type_at(self, key_path: &KeyPath) -> Option<TypeAtPath<'a>> {
        fn go<'a, 'b>(
            type_info: TypeAtPath<'a>,
            mut stack: Peekable<impl Iterator<Item = &'b Key>>,
        ) -> Option<TypeAtPath<'a>> {
            let head = stack.next()?;

            let value_at_key: TypeAtPath<'_> = match head {
                // .foo
                Key::Field(NamedOrNumbered::Named(key)) => match type_info {
                    TypeAtPath::Struct(struct_) => {
                        struct_.field_type(key)?.into_type_info_at_path()
                    }
                    TypeAtPath::Variant(variant) => match variant {
                        Variant::Struct(struct_variant) => {
                            struct_variant.field_type(key)?.into_type_info_at_path()
                        }
                        Variant::Tuple(_) | Variant::Unit(_) => return None,
                    },
                    TypeAtPath::Enum(_)
                    | TypeAtPath::TupleStruct(_)
                    | TypeAtPath::Tuple(_)
                    | TypeAtPath::List(_)
                    | TypeAtPath::Array(_)
                    | TypeAtPath::Map(_)
                    | TypeAtPath::Scalar(_)
                    | TypeAtPath::Opaque(_) => return None,
                },
                // .0
                Key::Field(NamedOrNumbered::Numbered(index)) => match type_info {
                    TypeAtPath::TupleStruct(tuple_struct) => {
                        tuple_struct.field_type_at(*index)?.into_type_info_at_path()
                    }
                    TypeAtPath::Tuple(tuple) => {
                        tuple.field_type_at(*index)?.into_type_info_at_path()
                    }
                    TypeAtPath::Variant(variant) => match variant {
                        Variant::Tuple(tuple) => {
                            tuple.field_type_at(*index)?.into_type_info_at_path()
                        }
                        Variant::Struct(_) | Variant::Unit(_) => return None,
                    },
                    TypeAtPath::Struct(_)
                    | TypeAtPath::Enum(_)
                    | TypeAtPath::List(_)
                    | TypeAtPath::Array(_)
                    | TypeAtPath::Map(_)
                    | TypeAtPath::Scalar(_)
                    | TypeAtPath::Opaque(_) => return None,
                },
                // ["foo"] or [0]
                Key::Get(key) => match type_info {
                    TypeAtPath::Map(map) => map.value_type().into_type_info_at_path(),
                    TypeAtPath::List(list) => {
                        if value_to_usize(key).is_some() {
                            list.element_type().into_type_info_at_path()
                        } else {
                            return None;
                        }
                    }
                    TypeAtPath::Array(array) => {
                        if value_to_usize(key).is_some() {
                            array.element_type().into_type_info_at_path()
                        } else {
                            return None;
                        }
                    }
                    TypeAtPath::Struct(_)
                    | TypeAtPath::TupleStruct(_)
                    | TypeAtPath::Tuple(_)
                    | TypeAtPath::Enum(_)
                    | TypeAtPath::Variant(_)
                    | TypeAtPath::Scalar(_)
                    | TypeAtPath::Opaque(_) => return None,
                },
                // ::Some
                Key::Variant(variant) => match type_info {
                    TypeAtPath::Enum(enum_) => enum_
                        .variants()
                        .find(|v| v.name() == variant)?
                        .into_type_info_at_path(),
                    TypeAtPath::Variant(v) => {
                        if v.name() == variant {
                            v.into_type_info_at_path()
                        } else {
                            return None;
                        }
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

            if stack.peek().is_none() {
                Some(value_at_key)
            } else {
                go(value_at_key, stack)
            }
        }

        if key_path.is_empty() {
            return Some(self);
        }

        go(self, key_path.path.iter().peekable())
    }
}
