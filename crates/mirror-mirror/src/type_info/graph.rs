use std::{
    any::{type_name, TypeId},
    collections::{hash_map::DefaultHasher, HashMap},
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
    Struct(StructInfoNode),
    TupleStruct(TupleStructInfoNode),
    Tuple(TupleInfoNode),
    Enum(EnumInfoNode),
    List(ListInfoNode),
    Array(ArrayInfoNode),
    Map(MapInfoNode),
    Scalar(ScalarInfoNode),
    Opaque,
}

macro_rules! impl_from {
    ($variant:ident($inner:ident)) => {
        impl From<$inner> for TypeInfoNode {
            fn from(inner: $inner) -> Self {
                Self::$variant(inner)
            }
        }
    };
}

impl_from! { Struct(StructInfoNode) }
impl_from! { TupleStruct(TupleStructInfoNode) }
impl_from! { Tuple(TupleInfoNode) }
impl_from! { Enum(EnumInfoNode) }
impl_from! { List(ListInfoNode) }
impl_from! { Array(ArrayInfoNode) }
impl_from! { Map(MapInfoNode) }
impl_from! { Scalar(ScalarInfoNode) }

#[derive(Debug, Clone, Serialize, Deserialize, Writable, Readable)]
pub struct StructInfoNode {
    pub(super) type_name: String,
    pub(super) fields: Vec<NamedFieldNode>,
    pub(super) metadata: Metadata,
    pub(super) docs: Box<[String]>,
}

impl StructInfoNode {
    pub fn new<T>(fields: &[NamedFieldNode], metadata: Metadata, docs: &[&'static str]) -> Self
    where
        T: Typed,
    {
        Self {
            type_name: type_name::<T>().to_owned(),
            fields: fields.to_vec(),
            metadata,
            docs: map_docs(docs),
        }
    }
}

fn map_docs(docs: &[&'static str]) -> Box<[String]> {
    docs.iter().map(|s| s.to_string()).collect()
}

#[derive(Debug, Clone, Serialize, Deserialize, Writable, Readable)]
pub struct TupleStructInfoNode {
    pub(super) type_name: String,
    pub(super) fields: Vec<UnnamedFieldNode>,
    pub(super) metadata: Metadata,
    pub(super) docs: Box<[String]>,
}

impl TupleStructInfoNode {
    pub fn new<T>(fields: &[UnnamedFieldNode], metadata: Metadata, docs: &[&'static str]) -> Self
    where
        T: Typed,
    {
        Self {
            type_name: type_name::<T>().to_owned(),
            fields: fields.to_vec(),
            metadata,
            docs: map_docs(docs),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Writable, Readable)]
pub struct EnumInfoNode {
    pub(super) type_name: String,
    pub(super) variants: Vec<VariantNode>,
    pub(super) metadata: Metadata,
    pub(super) docs: Box<[String]>,
}

impl EnumInfoNode {
    pub fn new<T>(variants: &[VariantNode], metadata: Metadata, docs: &[&'static str]) -> Self
    where
        T: Typed,
    {
        Self {
            type_name: type_name::<T>().to_owned(),
            variants: variants.to_vec(),
            metadata,
            docs: map_docs(docs),
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
    pub(super) docs: Box<[String]>,
}

impl StructVariantInfoNode {
    pub fn new(
        name: &'static str,
        fields: &[NamedFieldNode],
        metadata: Metadata,
        docs: &[&'static str],
    ) -> Self {
        Self {
            name: name.to_owned(),
            fields: fields.to_vec(),
            metadata,
            docs: map_docs(docs),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Writable, Readable)]
pub struct TupleVariantInfoNode {
    pub(super) name: String,
    pub(super) fields: Vec<UnnamedFieldNode>,
    pub(super) metadata: Metadata,
    pub(super) docs: Box<[String]>,
}

impl TupleVariantInfoNode {
    pub fn new(
        name: &'static str,
        fields: &[UnnamedFieldNode],
        metadata: Metadata,
        docs: &[&'static str],
    ) -> Self {
        Self {
            name: name.to_owned(),
            fields: fields.to_vec(),
            metadata,
            docs: map_docs(docs),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Writable, Readable)]
pub struct UnitVariantInfoNode {
    pub(super) name: String,
    pub(super) metadata: Metadata,
    pub(super) docs: Box<[String]>,
}

impl UnitVariantInfoNode {
    pub fn new(name: &'static str, metadata: Metadata, docs: &[&'static str]) -> Self {
        Self {
            name: name.to_owned(),
            metadata,
            docs: map_docs(docs),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Writable, Readable)]
pub struct TupleInfoNode {
    pub(super) type_name: String,
    pub(super) fields: Vec<UnnamedFieldNode>,
    pub(super) metadata: Metadata,
    pub(super) docs: Box<[String]>,
}

impl TupleInfoNode {
    pub fn new<T>(fields: &[UnnamedFieldNode], metadata: Metadata, docs: &[&'static str]) -> Self
    where
        T: Typed,
    {
        Self {
            type_name: type_name::<T>().to_owned(),
            fields: fields.to_vec(),
            metadata,
            docs: map_docs(docs),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Writable, Readable)]
pub struct NamedFieldNode {
    pub(super) name: String,
    pub(super) id: Id,
    pub(super) metadata: Metadata,
    pub(super) docs: Box<[String]>,
}

impl NamedFieldNode {
    pub fn new<T>(
        name: &'static str,
        metadata: Metadata,
        docs: &[&'static str],
        graph: &mut TypeInfoGraph,
    ) -> Self
    where
        T: Typed,
    {
        Self {
            name: name.to_owned(),
            id: T::build(graph),
            metadata,
            docs: map_docs(docs),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Writable, Readable)]
pub struct UnnamedFieldNode {
    pub(super) id: Id,
    pub(super) metadata: Metadata,
    pub(super) docs: Box<[String]>,
}

impl UnnamedFieldNode {
    pub fn new<T>(metadata: Metadata, docs: &[&'static str], graph: &mut TypeInfoGraph) -> Self
    where
        T: Typed,
    {
        Self {
            id: T::build(graph),
            metadata,
            docs: map_docs(docs),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Writable, Readable)]
pub struct ArrayInfoNode {
    pub(super) type_name: String,
    pub(super) field_type_id: Id,
    pub(super) len: usize,
}

impl ArrayInfoNode {
    pub(crate) fn new<L, T, const N: usize>(graph: &mut TypeInfoGraph) -> Self
    where
        L: Typed,
        T: Typed,
    {
        Self {
            type_name: type_name::<L>().to_owned(),
            field_type_id: T::build(graph),
            len: N,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Writable, Readable)]
pub struct ListInfoNode {
    pub(super) type_name: String,
    pub(super) field_type_id: Id,
}

impl ListInfoNode {
    pub(crate) fn new<L, T>(graph: &mut TypeInfoGraph) -> Self
    where
        L: Typed,
        T: Typed,
    {
        Self {
            type_name: type_name::<L>().to_owned(),
            field_type_id: T::build(graph),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Writable, Readable)]
pub struct MapInfoNode {
    pub(super) type_name: String,
    pub(super) key_type_id: Id,
    pub(super) value_type_id: Id,
}

impl MapInfoNode {
    pub(crate) fn new<M, K, V>(graph: &mut TypeInfoGraph) -> Self
    where
        M: Typed,
        K: Typed,
        V: Typed,
    {
        Self {
            type_name: type_name::<M>().to_owned(),
            key_type_id: K::build(graph),
            value_type_id: V::build(graph),
        }
    }
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
