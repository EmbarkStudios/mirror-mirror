use serde::{Deserialize, Serialize};
use speedy::{Readable, Writable};

use graph::*;

use crate::{
    key_path::{Key, KeyPath},
    Reflect,
};

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
    fn at_typed(self, key_path: KeyPath) -> Option<AtTyped<'a>>;
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
    fn at_typed(self, key_path: KeyPath) -> Option<AtTyped<'a>> {
        self.type_().at_typed(key_path)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum TypeInfo<'a> {
    Struct(Option<StructInfo<'a>>),
    TupleStruct(Option<TupleStructInfo<'a>>),
    Tuple(Option<TupleInfo<'a>>),
    Enum(Option<EnumInfo<'a>>),
    List(ListInfo<'a>),
    Map(MapInfo<'a>),
    Scalar(ScalarInfo),
    Opaque,
}

impl<'a> TypeInfo<'a> {
    fn new(id: Id, graph: &'a TypeInfoGraph) -> Self {
        match graph.get(id) {
            TypeInfoNode::Struct(node) => {
                let node = node.as_ref().map(|node| StructInfo { node, graph });
                TypeInfo::Struct(node)
            }
            TypeInfoNode::TupleStruct(node) => {
                let node = node.as_ref().map(|node| TupleStructInfo { node, graph });
                TypeInfo::TupleStruct(node)
            }
            TypeInfoNode::Tuple(node) => {
                let node = node.as_ref().map(|node| TupleInfo { node, graph });
                TypeInfo::Tuple(node)
            }
            TypeInfoNode::Enum(node) => {
                let node = node.as_ref().map(|node| EnumInfo { node, graph });
                TypeInfo::Enum(node)
            }
            TypeInfoNode::List(node) => {
                let node = ListInfo { node, graph };
                TypeInfo::List(node)
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
            TypeInfoNode::Opaque => TypeInfo::Opaque,
        }
    }

    pub fn type_name(self) -> Option<&'a str> {
        match self {
            TypeInfo::Struct(inner) => inner.map(|inner| inner.type_name()),
            TypeInfo::TupleStruct(inner) => inner.map(|inner| inner.type_name()),
            TypeInfo::Tuple(inner) => inner.map(|inner| inner.type_name()),
            TypeInfo::Enum(inner) => inner.map(|inner| inner.type_name()),
            TypeInfo::List(inner) => Some(inner.type_name()),
            TypeInfo::Map(inner) => Some(inner.type_name()),
            TypeInfo::Scalar(inner) => Some(inner.type_name()),
            TypeInfo::Opaque => None,
        }
    }

    fn into_at_typed(self) -> Option<AtTyped<'a>> {
        match self {
            TypeInfo::Struct(inner) => inner.map(|inner| inner.into_at_typed()),
            TypeInfo::TupleStruct(inner) => inner.map(|inner| inner.into_at_typed()),
            TypeInfo::Tuple(inner) => inner.map(|inner| inner.into_at_typed()),
            TypeInfo::Enum(inner) => inner.map(|inner| inner.into_at_typed()),
            TypeInfo::List(inner) => Some(inner.into_at_typed()),
            TypeInfo::Map(inner) => Some(inner.into_at_typed()),
            TypeInfo::Scalar(inner) => Some(inner.into_at_typed()),
            TypeInfo::Opaque => Some(AtTyped::Opaque),
        }
    }

    pub fn as_struct(self) -> Option<StructInfo<'a>> {
        match self {
            TypeInfo::Struct(inner) => inner,
            _ => None,
        }
    }

    pub fn as_tuple_struct(self) -> Option<TupleStructInfo<'a>> {
        match self {
            TypeInfo::TupleStruct(inner) => inner,
            _ => None,
        }
    }

    pub fn as_tuple(self) -> Option<TupleInfo<'a>> {
        match self {
            TypeInfo::Tuple(inner) => inner,
            _ => None,
        }
    }

    pub fn as_enum(self) -> Option<EnumInfo<'a>> {
        match self {
            TypeInfo::Enum(inner) => inner,
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
}

impl<'a> GetTypedPath<'a> for TypeInfo<'a> {
    fn at_typed(self, key_path: KeyPath) -> Option<AtTyped<'a>> {
        self.into_at_typed()?.at_typed(key_path)
    }
}

impl<'a> GetTypedPath<'a> for StructInfo<'a> {
    fn at_typed(self, key_path: KeyPath) -> Option<AtTyped<'a>> {
        self.into_at_typed().at_typed(key_path)
    }
}

impl<'a> GetTypedPath<'a> for TupleStructInfo<'a> {
    fn at_typed(self, key_path: KeyPath) -> Option<AtTyped<'a>> {
        self.into_at_typed().at_typed(key_path)
    }
}

impl<'a> GetTypedPath<'a> for TupleInfo<'a> {
    fn at_typed(self, key_path: KeyPath) -> Option<AtTyped<'a>> {
        self.into_at_typed().at_typed(key_path)
    }
}

impl<'a> GetTypedPath<'a> for EnumInfo<'a> {
    fn at_typed(self, key_path: KeyPath) -> Option<AtTyped<'a>> {
        self.into_at_typed().at_typed(key_path)
    }
}

impl<'a> GetTypedPath<'a> for ListInfo<'a> {
    fn at_typed(self, key_path: KeyPath) -> Option<AtTyped<'a>> {
        self.into_at_typed().at_typed(key_path)
    }
}

impl<'a> GetTypedPath<'a> for MapInfo<'a> {
    fn at_typed(self, key_path: KeyPath) -> Option<AtTyped<'a>> {
        self.into_at_typed().at_typed(key_path)
    }
}

impl GetTypedPath<'static> for ScalarInfo {
    fn at_typed(self, key_path: KeyPath) -> Option<AtTyped<'static>> {
        self.into_at_typed().at_typed(key_path)
    }
}

impl<'a> GetTypedPath<'a> for VariantInfo<'a> {
    fn at_typed(self, key_path: KeyPath) -> Option<AtTyped<'a>> {
        self.into_at_typed().at_typed(key_path)
    }
}

pub trait GetMeta<'a> {
    fn get_meta(self, key: &str) -> Option<&'a dyn Reflect>;
}

impl<'a> GetMeta<'a> for TypeInfo<'a> {
    fn get_meta(self, key: &str) -> Option<&'a dyn Reflect> {
        match self {
            TypeInfo::Struct(inner) => inner.as_ref().and_then(|inner| inner.get_meta(key)),
            TypeInfo::TupleStruct(inner) => inner.as_ref().and_then(|inner| inner.get_meta(key)),
            TypeInfo::Enum(inner) => inner.as_ref().and_then(|inner| inner.get_meta(key)),
            TypeInfo::Tuple(_)
            | TypeInfo::List(_)
            | TypeInfo::Map(_)
            | TypeInfo::Scalar(_)
            | TypeInfo::Opaque => None,
        }
    }
}

macro_rules! impl_get_meta {
    ($($ident:ident)*) => {
        $(
            impl<'a> GetMeta<'a> for $ident<'a> {
                fn get_meta(self, key: &str) -> Option<&'a dyn Reflect> {
                    Some(self.node.metadata.get(key)?.as_reflect())
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

    fn into_at_typed(self) -> AtTyped<'static> {
        AtTyped::Scalar(self)
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

    fn into_at_typed(self) -> AtTyped<'a> {
        AtTyped::Struct(self)
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

    fn into_at_typed(self) -> AtTyped<'a> {
        AtTyped::TupleStruct(self)
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

    fn into_at_typed(self) -> AtTyped<'a> {
        AtTyped::Tuple(self)
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
                graph: self.graph,
            }),
            VariantNode::Tuple(node) => VariantInfo::Tuple(TupleVariantInfo {
                node,
                graph: self.graph,
            }),
            VariantNode::Unit(node) => VariantInfo::Unit(UnitVariantInfo { node }),
        })
    }

    fn into_at_typed(self) -> AtTyped<'a> {
        AtTyped::Enum(self)
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

    fn into_at_typed(self) -> AtTyped<'a> {
        AtTyped::Variant(self)
    }
}

impl<'a> GetMeta<'a> for VariantInfo<'a> {
    fn get_meta(self, key: &str) -> Option<&'a dyn Reflect> {
        match self {
            VariantInfo::Struct(inner) => inner.get_meta(key),
            VariantInfo::Tuple(inner) => inner.get_meta(key),
            VariantInfo::Unit(inner) => inner.get_meta(key),
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
}

#[derive(Copy, Clone, Debug)]
pub struct StructVariantInfo<'a> {
    node: &'a StructVariantInfoNode,
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
}

#[derive(Copy, Clone, Debug)]
pub struct TupleVariantInfo<'a> {
    node: &'a TupleVariantInfoNode,
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
}

#[derive(Copy, Clone, Debug)]
pub struct UnitVariantInfo<'a> {
    node: &'a UnitVariantInfoNode,
}

impl<'a> UnitVariantInfo<'a> {
    pub fn name(self) -> &'a str {
        &self.node.name
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

    fn into_at_typed(self) -> AtTyped<'a> {
        AtTyped::List(self)
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

    fn into_at_typed(self) -> AtTyped<'a> {
        AtTyped::Map(self)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum AtTyped<'a> {
    Struct(StructInfo<'a>),
    TupleStruct(TupleStructInfo<'a>),
    Tuple(TupleInfo<'a>),
    Enum(EnumInfo<'a>),
    Variant(VariantInfo<'a>),
    List(ListInfo<'a>),
    Map(MapInfo<'a>),
    Scalar(ScalarInfo),
    Opaque,
}

impl<'a> AtTyped<'a> {
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
}

impl<'a> GetTypedPath<'a> for AtTyped<'a> {
    fn at_typed(self, key_path: KeyPath) -> Option<AtTyped<'a>> {
        fn go(type_info: AtTyped<'_>, mut stack: Vec<Key>) -> Option<AtTyped<'_>> {
            let head = stack.pop()?;

            let value_at_key: AtTyped<'_> = match head {
                Key::Field(key) => match type_info {
                    AtTyped::Struct(struct_) => struct_
                        .fields()
                        .find(|field| field.name() == key)?
                        .type_()
                        .into_at_typed()?,
                    AtTyped::Map(map) => map.value_type().into_at_typed()?,
                    AtTyped::Variant(variant) => match variant {
                        VariantInfo::Struct(struct_variant) => struct_variant
                            .fields()
                            .find(|field| field.name() == key)?
                            .type_()
                            .into_at_typed()?,
                        VariantInfo::Tuple(_) | VariantInfo::Unit(_) => return None,
                    },
                    AtTyped::Enum(enum_) => {
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
                                .into_at_typed()?
                        } else {
                            return None;
                        }
                    }
                    AtTyped::TupleStruct(_)
                    | AtTyped::Tuple(_)
                    | AtTyped::List(_)
                    | AtTyped::Scalar(_)
                    | AtTyped::Opaque => return None,
                },
                Key::Element(index) => match type_info {
                    AtTyped::List(list) => list.field_type().into_at_typed()?,
                    AtTyped::Map(map) => map.value_type().into_at_typed()?,
                    AtTyped::TupleStruct(tuple_struct) => {
                        tuple_struct.fields().nth(index)?.type_().into_at_typed()?
                    }
                    AtTyped::Tuple(tuple) => tuple.fields().nth(index)?.type_().into_at_typed()?,

                    AtTyped::Variant(variant) => match variant {
                        VariantInfo::Tuple(tuple_variant) => {
                            tuple_variant.fields().nth(index)?.type_().into_at_typed()?
                        }
                        VariantInfo::Struct(_) | VariantInfo::Unit(_) => return None,
                    },

                    AtTyped::Enum(_) | AtTyped::Scalar(_) | AtTyped::Struct(_) | AtTyped::Opaque => {
                        return None
                    }
                },
                Key::Variant(variant) => match type_info {
                    AtTyped::Variant(v) => {
                        if v.name() == variant {
                            AtTyped::Variant(v)
                        } else {
                            return None;
                        }
                    }
                    AtTyped::Enum(enum_) => {
                        let variant_info = enum_.variants().find(|v| v.name() == variant)?;
                        AtTyped::Variant(variant_info)
                    }
                    AtTyped::Struct(_)
                    | AtTyped::TupleStruct(_)
                    | AtTyped::Tuple(_)
                    | AtTyped::List(_)
                    | AtTyped::Map(_)
                    | AtTyped::Scalar(_)
                    | AtTyped::Opaque => return None,
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
    use crate as mirror_mirror;
    use crate::Reflect;

    #[test]
    fn struct_() {
        #[derive(Reflect, Clone, Debug)]
        struct Foo {
            n: i32,
            foos: Vec<Foo>,
        }

        let type_info = <Foo as Typed>::type_info();

        assert_eq!(
            type_info.type_().type_name().unwrap(),
            "mirror_mirror::type_info::tests::struct_::Foo"
        );

        let struct_ = match type_info.type_() {
            TypeInfo::Struct(Some(struct_)) => struct_,
            _ => panic!("wat"),
        };

        assert_eq!(
            struct_.type_name(),
            "mirror_mirror::type_info::tests::struct_::Foo"
        );

        for field in struct_.fields() {
            match field.name() {
                "foos" => {
                    assert_eq!(
                        field.type_().type_name().unwrap(),
                        "alloc::vec::Vec<mirror_mirror::type_info::tests::struct_::Foo>"
                    );

                    let list = match field.type_() {
                        TypeInfo::List(list) => list,
                        _ => panic!("wat"),
                    };

                    assert_eq!(
                        list.type_name(),
                        "alloc::vec::Vec<mirror_mirror::type_info::tests::struct_::Foo>"
                    );

                    assert_eq!(
                        list.field_type().type_name().unwrap(),
                        "mirror_mirror::type_info::tests::struct_::Foo"
                    );
                }
                "n" => {
                    assert_eq!(field.type_().type_name().unwrap(), "i32");
                    let scalar = match field.type_() {
                        TypeInfo::Scalar(scalar) => scalar,
                        _ => panic!("wat"),
                    };
                    assert_eq!(scalar.type_name(), "i32");
                }
                _ => panic!("wat"),
            }
        }
    }

    #[test]
    fn enum_() {
        #[derive(Reflect, Clone, Debug)]
        enum Foo {
            A { a: String },
            B(Vec<Foo>),
            C,
        }
    }
}
