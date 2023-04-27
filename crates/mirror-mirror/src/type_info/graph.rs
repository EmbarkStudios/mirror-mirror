use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::type_name;
use core::any::TypeId;
use core::hash::BuildHasher;
use core::hash::Hash;
use core::hash::Hasher;
use core::ops::Deref;

use tame_containers::UnorderedMap;

use super::*;
use crate::Value;

/// A `TypeGraph`'s node that refers to a specific type via its `TypeId'.
#[derive(Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Debug)]
#[cfg_attr(feature = "speedy", derive(speedy::Readable, speedy::Writable))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NodeId(u64);

impl Hash for NodeId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.0);
    }
}

impl NodeId {
    fn new<T>() -> Self
    where
        T: 'static,
    {
        let mut hasher = STATIC_RANDOM_STATE.build_hasher();
        TypeId::of::<T>().hash(&mut hasher);
        Self(hasher.finish())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub(super) struct WithId<T> {
    pub(super) id: NodeId,
    inner: T,
}

impl<T> WithId<T> {
    pub(super) fn new(id: NodeId, inner: T) -> Self {
        Self { id, inner }
    }
}

impl<T> Deref for WithId<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// A hasher that does no hashing because we already have a u64 hash that should be
/// just as well distributed.
#[derive(Clone, Copy)]
pub(crate) struct NoHashHasher(u64);

impl Default for NoHashHasher {
    fn default() -> Self {
        Self(0)
    }
}

impl Hasher for NoHashHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, _: &[u8]) {
        panic!("Only write_u64 should be called exactly once when using NoHashHasher! This is a bug, please report it.")
    }

    fn write_u64(&mut self, v: u64) {
        self.0 = v;
    }
}

#[derive(Default, Clone, Copy)]
pub(crate) struct BuildNoHashHasher;

impl BuildHasher for BuildNoHashHasher {
    type Hasher = NoHashHasher;

    #[inline(always)]
    fn build_hasher(&self) -> Self::Hasher {
        NoHashHasher(0)
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "speedy", derive(speedy::Readable, speedy::Writable))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TypeGraph {
    pub(super) map: UnorderedMap<NodeId, Option<TypeNode>, BuildNoHashHasher>,
}

impl TypeGraph {
    pub(super) fn get(&self, id: NodeId) -> &TypeNode {
        const ERROR: &str = "no node found in graph. This is a bug. Please open an issue.";
        self.map.get(&id).expect(ERROR).as_ref().expect(ERROR)
    }

    pub fn get_or_build_node_with<T, I>(&mut self, f: impl FnOnce(&mut Self) -> I) -> NodeId
    where
        I: Into<TypeNode>,
        T: DescribeType,
    {
        let id = NodeId::new::<T>();
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "speedy", derive(speedy::Readable, speedy::Writable))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TypeNode {
    Struct(StructNode),
    TupleStruct(TupleStructNode),
    Tuple(TupleNode),
    Enum(EnumNode),
    List(ListNode),
    Array(ArrayNode),
    Map(MapNode),
    Scalar(ScalarNode),
    Opaque(OpaqueNode),
}

macro_rules! impl_from {
    ($variant:ident($inner:ident)) => {
        impl From<$inner> for TypeNode {
            fn from(inner: $inner) -> Self {
                Self::$variant(inner)
            }
        }
    };
}

impl_from! { Struct(StructNode) }
impl_from! { TupleStruct(TupleStructNode) }
impl_from! { Tuple(TupleNode) }
impl_from! { Enum(EnumNode) }
impl_from! { List(ListNode) }
impl_from! { Array(ArrayNode) }
impl_from! { Map(MapNode) }
impl_from! { Scalar(ScalarNode) }
impl_from! { Opaque(OpaqueNode) }

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "speedy", derive(speedy::Readable, speedy::Writable))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StructNode {
    pub(super) type_name: String,
    pub(super) fields: BTreeMap<String, NamedFieldNode>,
    pub(super) field_names: Box<[String]>,
    pub(super) metadata: BTreeMap<String, Value>,
    pub(super) docs: Box<[String]>,
}

impl StructNode {
    pub fn new<T>(
        fields: &[NamedFieldNode],
        metadata: BTreeMap<&'static str, Value>,
        docs: &[&'static str],
    ) -> Self
    where
        T: DescribeType,
    {
        Self {
            type_name: type_name::<T>().to_owned(),
            fields: fields
                .iter()
                .map(|field| (field.name.clone(), field.clone()))
                .collect(),
            field_names: fields.iter().map(|field| field.name.clone()).collect(),
            metadata: map_metadata(metadata),
            docs: map_docs(docs),
        }
    }
}

fn map_metadata(metadata: BTreeMap<&'static str, Value>) -> BTreeMap<String, Value> {
    metadata
        .into_iter()
        .map(|(key, value)| (key.to_owned(), value))
        .collect()
}

fn map_docs(docs: &[&'static str]) -> Box<[String]> {
    docs.iter().map(|s| (*s).to_owned()).collect()
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "speedy", derive(speedy::Readable, speedy::Writable))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TupleStructNode {
    pub(super) type_name: String,
    pub(super) fields: Vec<UnnamedFieldNode>,
    pub(super) metadata: BTreeMap<String, Value>,
    pub(super) docs: Box<[String]>,
}

impl TupleStructNode {
    pub fn new<T>(
        fields: &[UnnamedFieldNode],
        metadata: BTreeMap<&'static str, Value>,
        docs: &[&'static str],
    ) -> Self
    where
        T: DescribeType,
    {
        Self {
            type_name: type_name::<T>().to_owned(),
            fields: fields.to_vec(),
            metadata: map_metadata(metadata),
            docs: map_docs(docs),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "speedy", derive(speedy::Readable, speedy::Writable))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EnumNode {
    pub(super) type_name: String,
    pub(super) variants: Vec<VariantNode>,
    pub(super) metadata: BTreeMap<String, Value>,
    pub(super) docs: Box<[String]>,
}

impl EnumNode {
    pub fn new<T>(
        variants: &[VariantNode],
        metadata: BTreeMap<&'static str, Value>,
        docs: &[&'static str],
    ) -> Self
    where
        T: DescribeType,
    {
        Self {
            type_name: type_name::<T>().to_owned(),
            variants: variants.to_vec(),
            metadata: map_metadata(metadata),
            docs: map_docs(docs),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "speedy", derive(speedy::Readable, speedy::Writable))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum VariantNode {
    Struct(StructVariantNode),
    Tuple(TupleVariantNode),
    Unit(UnitVariantNode),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "speedy", derive(speedy::Readable, speedy::Writable))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StructVariantNode {
    pub(super) name: String,
    pub(super) fields: BTreeMap<String, NamedFieldNode>,
    pub(super) field_names: Box<[String]>,
    pub(super) metadata: BTreeMap<String, Value>,
    pub(super) docs: Box<[String]>,
}

impl StructVariantNode {
    pub fn new(
        name: &'static str,
        fields: &[NamedFieldNode],
        metadata: BTreeMap<&'static str, Value>,
        docs: &[&'static str],
    ) -> Self {
        Self {
            name: name.to_owned(),
            fields: fields
                .iter()
                .map(|field| (field.name.clone(), field.clone()))
                .collect(),
            field_names: fields.iter().map(|field| field.name.clone()).collect(),
            metadata: map_metadata(metadata),
            docs: map_docs(docs),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "speedy", derive(speedy::Readable, speedy::Writable))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TupleVariantNode {
    pub(super) name: String,
    pub(super) fields: Vec<UnnamedFieldNode>,
    pub(super) metadata: BTreeMap<String, Value>,
    pub(super) docs: Box<[String]>,
}

impl TupleVariantNode {
    pub fn new(
        name: &'static str,
        fields: &[UnnamedFieldNode],
        metadata: BTreeMap<&'static str, Value>,
        docs: &[&'static str],
    ) -> Self {
        Self {
            name: name.to_owned(),
            fields: fields.to_vec(),
            metadata: map_metadata(metadata),
            docs: map_docs(docs),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "speedy", derive(speedy::Readable, speedy::Writable))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UnitVariantNode {
    pub(super) name: String,
    pub(super) metadata: BTreeMap<String, Value>,
    pub(super) docs: Box<[String]>,
}

impl UnitVariantNode {
    pub fn new(
        name: &'static str,
        metadata: BTreeMap<&'static str, Value>,
        docs: &[&'static str],
    ) -> Self {
        Self {
            name: name.to_owned(),
            metadata: map_metadata(metadata),
            docs: map_docs(docs),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "speedy", derive(speedy::Readable, speedy::Writable))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TupleNode {
    pub(super) type_name: String,
    pub(super) fields: Vec<UnnamedFieldNode>,
    pub(super) metadata: BTreeMap<String, Value>,
    pub(super) docs: Box<[String]>,
}

impl TupleNode {
    pub fn new<T>(
        fields: &[UnnamedFieldNode],
        metadata: BTreeMap<&'static str, Value>,
        docs: &[&'static str],
    ) -> Self
    where
        T: DescribeType,
    {
        Self {
            type_name: type_name::<T>().to_owned(),
            fields: fields.to_vec(),
            metadata: map_metadata(metadata),
            docs: map_docs(docs),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "speedy", derive(speedy::Readable, speedy::Writable))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NamedFieldNode {
    pub(super) name: String,
    pub(super) id: NodeId,
    pub(super) metadata: BTreeMap<String, Value>,
    pub(super) docs: Box<[String]>,
}

impl NamedFieldNode {
    pub fn new<T>(
        name: &'static str,
        metadata: BTreeMap<&'static str, Value>,
        docs: &[&'static str],
        graph: &mut TypeGraph,
    ) -> Self
    where
        T: DescribeType,
    {
        Self {
            name: name.to_owned(),
            id: T::build(graph),
            metadata: map_metadata(metadata),
            docs: map_docs(docs),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "speedy", derive(speedy::Readable, speedy::Writable))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UnnamedFieldNode {
    pub(super) id: NodeId,
    pub(super) metadata: BTreeMap<String, Value>,
    pub(super) docs: Box<[String]>,
}

impl UnnamedFieldNode {
    pub fn new<T>(
        metadata: BTreeMap<&'static str, Value>,
        docs: &[&'static str],
        graph: &mut TypeGraph,
    ) -> Self
    where
        T: DescribeType,
    {
        Self {
            id: T::build(graph),
            metadata: map_metadata(metadata),
            docs: map_docs(docs),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "speedy", derive(speedy::Readable, speedy::Writable))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ArrayNode {
    pub(super) type_name: String,
    pub(super) field_type_id: NodeId,
    pub(super) len: usize,
}

impl ArrayNode {
    pub(crate) fn new<L, T, const N: usize>(graph: &mut TypeGraph) -> Self
    where
        L: DescribeType,
        T: DescribeType,
    {
        Self {
            type_name: type_name::<L>().to_owned(),
            field_type_id: T::build(graph),
            len: N,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "speedy", derive(speedy::Readable, speedy::Writable))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ListNode {
    pub(super) type_name: String,
    pub(super) field_type_id: NodeId,
}

impl ListNode {
    pub(crate) fn new<L, T>(graph: &mut TypeGraph) -> Self
    where
        L: DescribeType,
        T: DescribeType,
    {
        Self {
            type_name: type_name::<L>().to_owned(),
            field_type_id: T::build(graph),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "speedy", derive(speedy::Readable, speedy::Writable))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MapNode {
    pub(super) type_name: String,
    pub(super) key_type_id: NodeId,
    pub(super) value_type_id: NodeId,
}

impl MapNode {
    pub(crate) fn new<M, K, V>(graph: &mut TypeGraph) -> Self
    where
        M: DescribeType,
        K: DescribeType,
        V: DescribeType,
    {
        Self {
            type_name: type_name::<M>().to_owned(),
            key_type_id: K::build(graph),
            value_type_id: V::build(graph),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
#[cfg_attr(feature = "speedy", derive(speedy::Readable, speedy::Writable))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ScalarNode {
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
            impl DescribeType for $ty {
                fn build(graph: &mut TypeGraph) -> NodeId {
                    graph.get_or_build_node_with::<Self, _>(|_graph| ScalarNode::$ty)
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "speedy", derive(speedy::Readable, speedy::Writable))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OpaqueNode {
    pub(super) type_name: String,
    pub(super) metadata: BTreeMap<String, Value>,
    pub(super) default_value: Option<Value>,
}

impl OpaqueNode {
    pub fn new<T>(metadata: BTreeMap<&'static str, Value>, _graph: &mut TypeGraph) -> Self
    where
        T: DescribeType,
    {
        Self {
            type_name: type_name::<T>().to_owned(),
            metadata: map_metadata(metadata),
            default_value: None,
        }
    }

    pub fn default_value(mut self, default_value: impl Into<Value>) -> Self {
        self.default_value = Some(default_value.into());
        self
    }
}
