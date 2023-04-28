use alloc::boxed::Box;
use alloc::string::String;
use core::any::Any;
use core::fmt;

use tame_containers::OrderedMap;

use crate::iter::PairIterMut;
use crate::type_info::graph::NodeId;
use crate::type_info::graph::OpaqueNode;
use crate::type_info::graph::TypeGraph;
use crate::DescribeType;
use crate::FromReflect;
use crate::Reflect;
use crate::ReflectMut;
use crate::ReflectOwned;
use crate::ReflectRef;
use crate::Value;

pub type FieldsIter<'a> = Box<dyn Iterator<Item = (&'a str, &'a dyn Reflect)> + 'a>;
pub type FieldsIterMut<'a> = Box<dyn Iterator<Item = (&'a str, &'a mut dyn Reflect)> + 'a>;

/// A reflected struct type.
///
/// Will be implemented by `#[derive(Reflect)]` on structs.
pub trait Struct: Reflect {
    fn field(&self, name: &str) -> Option<&dyn Reflect>;

    fn field_mut(&mut self, name: &str) -> Option<&mut dyn Reflect>;

    fn field_at(&self, index: usize) -> Option<&dyn Reflect>;

    fn field_at_mut(&mut self, index: usize) -> Option<&mut dyn Reflect>;

    fn name_at(&self, index: usize) -> Option<&str>;

    fn fields(&self) -> FieldsIter<'_>;

    fn fields_mut(&mut self) -> FieldsIterMut<'_>;

    fn fields_len(&self) -> usize;
}

impl fmt::Debug for dyn Struct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_reflect().debug(f)
    }
}

#[derive(Default, Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[cfg_attr(feature = "speedy", derive(speedy::Readable, speedy::Writable))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StructValue {
    fields: OrderedMap<String, Value>,
}

impl StructValue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            fields: OrderedMap::with_capacity(capacity),
        }
    }

    pub fn with_field(mut self, name: impl Into<String>, value: impl Into<Value>) -> Self {
        self.set_field(name, value);
        self
    }

    pub fn set_field(&mut self, name: impl Into<String>, value: impl Into<Value>) {
        let name = name.into();
        self.fields.insert(name, value.into());
    }
}

impl DescribeType for StructValue {
    fn build(graph: &mut TypeGraph) -> NodeId {
        graph.get_or_build_node_with::<Self, _>(|graph| {
            OpaqueNode::new::<Self>(Default::default(), graph)
        })
    }
}

impl Reflect for StructValue {
    trivial_reflect_methods!();

    fn patch(&mut self, value: &dyn Reflect) {
        if let Some(struct_) = value.reflect_ref().as_struct() {
            for (name, value) in self.fields_mut() {
                if let Some(new_value) = struct_.field(name) {
                    value.patch(new_value);
                }
            }
        }
    }

    fn to_value(&self) -> Value {
        self.clone().into()
    }

    fn clone_reflect(&self) -> Box<dyn Reflect> {
        Box::new(self.clone())
    }

    fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "{self:#?}")
        } else {
            write!(f, "{self:?}")
        }
    }

    fn reflect_owned(self: Box<Self>) -> ReflectOwned {
        ReflectOwned::Struct(self)
    }

    fn reflect_ref(&self) -> ReflectRef<'_> {
        ReflectRef::Struct(self)
    }

    fn reflect_mut(&mut self) -> ReflectMut<'_> {
        ReflectMut::Struct(self)
    }
}

impl Struct for StructValue {
    fn field(&self, name: &str) -> Option<&dyn Reflect> {
        Some(self.fields.get(name)?)
    }

    fn field_mut(&mut self, name: &str) -> Option<&mut dyn Reflect> {
        Some(self.fields.get_mut(name)?)
    }

    fn fields(&self) -> FieldsIter<'_> {
        let iter = self
            .fields
            .iter()
            .map(|(key, value)| (&**key, value.as_reflect()));
        Box::new(iter)
    }

    fn fields_mut(&mut self) -> PairIterMut<'_> {
        let iter = self
            .fields
            .iter_mut()
            .map(|(key, value)| (&**key, value.as_reflect_mut()));
        Box::new(iter)
    }

    fn fields_len(&self) -> usize {
        self.fields.len()
    }

    fn field_at(&self, index: usize) -> Option<&dyn Reflect> {
        let (_name, value) = self.fields.get_index(index)?;
        Some(value)
    }

    fn name_at(&self, index: usize) -> Option<&str> {
        let (name, _value) = self.fields.get_index(index)?;
        Some(name.as_str())
    }

    fn field_at_mut(&mut self, index: usize) -> Option<&mut dyn Reflect> {
        let (_name, value) = self.fields.get_index_mut(index)?;
        Some(value)
    }
}

impl FromReflect for StructValue {
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
        let struct_ = reflect.reflect_ref().as_struct()?;
        let this = struct_
            .fields()
            .fold(StructValue::default(), |builder, (name, value)| {
                builder.with_field(name, value.to_value())
            });
        Some(this)
    }
}

impl<S, V> FromIterator<(S, V)> for StructValue
where
    S: Into<String>,
    V: Reflect,
{
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (S, V)>,
    {
        let fields = OrderedMap::from_iter(iter.into_iter().map(|(k, v)| (k.into(), v.to_value())));
        Self { fields }
    }
}
