use serde::{Deserialize, Serialize};
use speedy::{Readable, Writable};

use graph::*;

use crate::{
    key_path::{Key, KeyPath},
    Reflect,
};

pub trait Typed: 'static {
    fn type_info() -> TypeInfoRoot {
        let mut graph = TypeInfoGraph::default();
        let id = Self::build(&mut graph);
        TypeInfoRoot { root: id, graph }
    }

    fn build(graph: &mut TypeInfoGraph) -> Id;
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

    pub fn at(&self, key_path: KeyPath) -> Option<TypeInfo<'_>> {
        self.type_().at(key_path)
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

    // TODO(david): make trait implemented for all type infos
    pub fn at(self, key_path: KeyPath) -> Option<TypeInfo<'a>> {
        fn go(type_info: TypeInfo<'_>, mut stack: Vec<Key>) -> Option<TypeInfo<'_>> {
            let head = stack.pop()?;

            let value_at_key = match head {
                Key::Field(key) => match type_info {
                    TypeInfo::Struct(inner) => {
                        inner?.fields().find(|field| field.name() == key)?.type_()
                    }
                    TypeInfo::Map(inner) => inner.value_type(),

                    TypeInfo::Enum(_) => todo!("enum"),

                    TypeInfo::TupleStruct(_)
                    | TypeInfo::Tuple(_)
                    | TypeInfo::List(_)
                    | TypeInfo::Scalar(_)
                    | TypeInfo::Opaque => return None,
                },
                Key::Element(index) => match type_info {
                    TypeInfo::List(inner) => inner.field_type(),
                    TypeInfo::TupleStruct(inner) => inner?.fields().nth(index)?.type_(),
                    TypeInfo::Tuple(inner) => inner?.fields().nth(index)?.type_(),

                    TypeInfo::Enum(_) => todo!("enum"),

                    TypeInfo::Scalar(_)
                    | TypeInfo::Struct(_)
                    | TypeInfo::Map(_)
                    | TypeInfo::Opaque => return None,
                },
                Key::Variant(variant) => match type_info {
                    TypeInfo::Enum(inner) => {
                        let matching_variant: VariantInfo =
                            inner?.variants().find(|v| v.name() == variant)?;
                        // let x = EnumInfo {
                        //     node,
                        //     graph: enum_info.graph,
                        // };
                        todo!()
                    }
                    TypeInfo::Struct(_)
                    | TypeInfo::TupleStruct(_)
                    | TypeInfo::Tuple(_)
                    | TypeInfo::List(_)
                    | TypeInfo::Map(_)
                    | TypeInfo::Scalar(_)
                    | TypeInfo::Opaque => return None,
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
}

pub mod graph {
    use std::{
        any::{type_name, TypeId},
        collections::{hash_map::DefaultHasher, BTreeMap, HashMap},
        hash::{Hash, Hasher},
    };

    use crate::Value;

    use super::*;

    #[derive(
        Clone,
        Copy,
        Serialize,
        Deserialize,
        Writable,
        Readable,
        Hash,
        PartialEq,
        PartialOrd,
        Ord,
        Eq,
        Debug,
    )]
    pub struct Id(u64);

    impl Id {
        pub fn new<T>() -> Self
        where
            T: 'static,
        {
            let mut hasher = DefaultHasher::default();
            TypeId::of::<T>().hash(&mut hasher);
            Self(hasher.finish())
        }
    }

    #[derive(Default, Debug, Clone, Serialize, Deserialize, Writable, Readable)]
    pub struct TypeInfoGraph {
        pub(super) map: HashMap<Id, Option<TypeInfoNode>>,
    }

    impl TypeInfoGraph {
        pub(super) fn get(&self, id: Id) -> &TypeInfoNode {
            const ERROR: &str = "no node found in graph. This is a bug. Please open an issue.";
            self.map.get(&id).expect(ERROR).as_ref().expect(ERROR)
        }

        pub fn get_or_build_with<T, I>(&mut self, f: impl FnOnce(&mut Self) -> I) -> Id
        where
            I: Into<TypeInfoNode>,
            T: Typed,
        {
            let id = Id::new::<T>();
            match self.map.get(&id) {
                // the data is already there
                Some(Some(_)) => id,
                // someone else is currently inserting the data
                Some(None) => id,
                // the data isn't there yet
                None => {
                    self.map.insert(id, None);
                    let info = f(self).into();
                    self.map.insert(id, Some(info));
                    id
                }
            }
        }
    }

    type Metadata = HashMap<String, Value>;

    #[derive(Debug, Clone, Serialize, Deserialize, Writable, Readable)]
    pub enum TypeInfoNode {
        Struct(Option<StructInfoNode>),
        TupleStruct(Option<TupleStructInfoNode>),
        Tuple(Option<TupleInfoNode>),
        Enum(Option<EnumInfoNode>),
        List(ListInfoNode),
        Map(MapInfoNode),
        Scalar(ScalarInfoNode),
        Opaque,
    }

    macro_rules! impl_from {
        ($variant:ident(Option<$inner:ident>)) => {
            impl From<$inner> for TypeInfoNode {
                fn from(inner: $inner) -> Self {
                    Self::$variant(Some(inner))
                }
            }
        };

        ($variant:ident($inner:ident)) => {
            impl From<$inner> for TypeInfoNode {
                fn from(inner: $inner) -> Self {
                    Self::$variant(inner)
                }
            }
        };
    }

    impl_from! { Struct(Option<StructInfoNode>) }
    impl_from! { TupleStruct(Option<TupleStructInfoNode>) }
    impl_from! { Tuple(Option<TupleInfoNode>) }
    impl_from! { Enum(Option<EnumInfoNode>) }
    impl_from! { List(ListInfoNode) }
    impl_from! { Map(MapInfoNode) }
    impl_from! { Scalar(ScalarInfoNode) }

    #[derive(Debug, Clone, Serialize, Deserialize, Writable, Readable)]
    pub struct StructInfoNode {
        pub(super) type_name: String,
        pub(super) fields: Vec<NamedFieldNode>,
        pub(super) metadata: Metadata,
    }

    impl StructInfoNode {
        pub fn new<T>(fields: &[NamedFieldNode], metadata: Metadata) -> Self
        where
            T: Typed,
        {
            Self {
                type_name: type_name::<T>().to_owned(),
                fields: fields.to_vec(),
                metadata,
            }
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Writable, Readable)]
    pub struct TupleStructInfoNode {
        pub(super) type_name: String,
        pub(super) fields: Vec<UnnamedFieldNode>,
        pub(super) metadata: Metadata,
    }

    impl TupleStructInfoNode {
        pub fn new<T>(fields: &[UnnamedFieldNode], metadata: Metadata) -> Self
        where
            T: Typed,
        {
            Self {
                type_name: type_name::<T>().to_owned(),
                fields: fields.to_vec(),
                metadata,
            }
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Writable, Readable)]
    pub struct EnumInfoNode {
        pub(super) type_name: String,
        pub(super) variants: Vec<VariantNode>,
        pub(super) metadata: Metadata,
    }

    impl EnumInfoNode {
        pub fn new<T>(variants: &[VariantNode], metadata: Metadata) -> Self
        where
            T: Typed,
        {
            Self {
                type_name: type_name::<T>().to_owned(),
                variants: variants.to_vec(),
                metadata,
            }
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Writable, Readable)]
    pub enum VariantNode {
        Struct(StructVariantInfoNode),
        Tuple(TupleVariantInfoNode),
        Unit(UnitVariantInfoNode),
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Writable, Readable)]
    pub struct StructVariantInfoNode {
        pub(super) name: String,
        pub(super) fields: Vec<NamedFieldNode>,
        pub(super) metadata: Metadata,
    }

    impl StructVariantInfoNode {
        pub fn new(name: &'static str, fields: &[NamedFieldNode], metadata: Metadata) -> Self {
            Self {
                name: name.to_owned(),
                fields: fields.to_vec(),
                metadata,
            }
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Writable, Readable)]
    pub struct TupleVariantInfoNode {
        pub(super) name: String,
        pub(super) fields: Vec<UnnamedFieldNode>,
        pub(super) metadata: Metadata,
    }

    impl TupleVariantInfoNode {
        pub fn new(name: &'static str, fields: &[UnnamedFieldNode], metadata: Metadata) -> Self {
            Self {
                name: name.to_owned(),
                fields: fields.to_vec(),
                metadata,
            }
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Writable, Readable)]
    pub struct UnitVariantInfoNode {
        pub(super) name: String,
        pub(super) metadata: Metadata,
    }

    impl UnitVariantInfoNode {
        pub fn new(name: &'static str, metadata: Metadata) -> Self {
            Self {
                name: name.to_owned(),
                metadata,
            }
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Writable, Readable)]
    pub struct TupleInfoNode {
        pub(super) type_name: String,
        pub(super) fields: Vec<UnnamedFieldNode>,
        pub(super) metadata: Metadata,
    }

    impl TupleInfoNode {
        pub fn new<T>(fields: &[UnnamedFieldNode], metadata: Metadata) -> Self
        where
            T: Typed,
        {
            Self {
                type_name: type_name::<T>().to_owned(),
                fields: fields.to_vec(),
                metadata,
            }
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Writable, Readable)]
    pub struct NamedFieldNode {
        pub(super) name: String,
        pub(super) id: Id,
        pub(super) metadata: Metadata,
    }

    impl NamedFieldNode {
        pub fn new<T>(name: &'static str, metadata: Metadata, graph: &mut TypeInfoGraph) -> Self
        where
            T: Typed,
        {
            Self {
                name: name.to_owned(),
                id: T::build(graph),
                metadata,
            }
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Writable, Readable)]
    pub struct UnnamedFieldNode {
        pub(super) id: Id,
        pub(super) metadata: Metadata,
    }

    impl UnnamedFieldNode {
        pub fn new<T>(metadata: Metadata, graph: &mut TypeInfoGraph) -> Self
        where
            T: Typed,
        {
            Self {
                id: T::build(graph),
                metadata,
            }
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Writable, Readable)]
    pub struct ListInfoNode {
        pub(super) type_name: String,
        pub(super) field_type_id: Id,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Writable, Readable)]
    pub struct MapInfoNode {
        pub(super) type_name: String,
        pub(super) key_type_id: Id,
        pub(super) value_type_id: Id,
    }

    #[derive(Debug, Clone)]
    #[allow(non_camel_case_types)]
    #[derive(Serialize, Deserialize, Writable, Readable)]
    pub enum ScalarInfoNode {
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

    macro_rules! scalar_typed {
    ($($ty:ident)*) => {
        $(
            impl Typed for $ty {
                fn build(graph: &mut TypeInfoGraph) -> Id {
                    graph.get_or_build_with::<Self, _>(|_graph| ScalarInfoNode::$ty)
                }
            }
        )*
    };
}

    scalar_typed! {
        usize u8 u16 u32 u64 u128
        i8 i16 i32 i64 i128
        f32 f64
        bool char String
    }

    impl<T> Typed for Vec<T>
    where
        T: Typed,
    {
        fn build(graph: &mut TypeInfoGraph) -> Id {
            graph.get_or_build_with::<Self, _>(|graph| ListInfoNode {
                type_name: type_name::<Self>().to_owned(),
                field_type_id: T::build(graph),
            })
        }
    }

    impl<K, V> Typed for BTreeMap<K, V>
    where
        K: Typed,
        V: Typed,
    {
        fn build(graph: &mut TypeInfoGraph) -> Id {
            graph.get_or_build_with::<Self, _>(|graph| MapInfoNode {
                type_name: type_name::<Self>().to_owned(),
                key_type_id: K::build(graph),
                value_type_id: V::build(graph),
            })
        }
    }

    impl<T> Typed for Option<T>
    where
        T: Typed,
    {
        fn build(graph: &mut TypeInfoGraph) -> Id {
            graph.get_or_build_with::<Self, _>(|graph| {
                EnumInfoNode::new::<Self>(
                    &[
                        VariantNode::Tuple(TupleVariantInfoNode::new(
                            "Some",
                            &[UnnamedFieldNode::new::<T>(Default::default(), graph)],
                            Default::default(),
                        )),
                        VariantNode::Unit(UnitVariantInfoNode::new("None", Default::default())),
                    ],
                    Default::default(),
                )
            })
        }
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
