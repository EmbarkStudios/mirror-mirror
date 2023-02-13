use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;
use core::fmt;
use core::iter::FusedIterator;

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
use crate::TypeDescriptor;
use crate::Value;

/// A reflected struct type.
///
/// Will be implemented by `#[derive(Reflect)]` on structs.
pub trait Struct: Reflect {
    fn field(&self, name: &str) -> Option<&dyn Reflect>;

    fn field_mut(&mut self, name: &str) -> Option<&mut dyn Reflect>;

    fn field_at(&self, index: usize) -> Option<&dyn Reflect>;

    fn field_at_mut(&mut self, index: usize) -> Option<&mut dyn Reflect>;

    fn name_at(&self, index: usize) -> Option<&str>;

    fn fields(&self) -> Iter<'_>;

    fn fields_mut(&mut self) -> PairIterMut<'_>;

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
    field_names: Vec<String>,
    // use a `BTreeMap` because `HashMap` isn't `serde::Serialize`
    fields: BTreeMap<String, Value>,
}

impl StructValue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_field(mut self, name: impl Into<String>, value: impl Into<Value>) -> Self {
        self.set_field(name, value);
        self
    }

    pub fn set_field(&mut self, name: impl Into<String>, value: impl Into<Value>) {
        let name = name.into();
        self.field_names.push(name.clone());
        self.fields.insert(name, value.into());
    }
}

impl Reflect for StructValue {
    fn type_descriptor(&self) -> Cow<'static, TypeDescriptor> {
        impl DescribeType for StructValue {
            fn build(graph: &mut TypeGraph) -> NodeId {
                graph.get_or_build_node_with::<Self, _>(|graph| {
                    OpaqueNode::new::<Self>(Default::default(), graph)
                })
            }
        }
        <Self as DescribeType>::type_descriptor()
    }

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

    fn fields(&self) -> Iter<'_> {
        Iter::new(self)
    }

    fn fields_mut(&mut self) -> PairIterMut<'_> {
        let iter = self
            .fields
            .iter_mut()
            .map(|(key, value)| (&**key, value.as_reflect_mut()));
        Box::new(iter)
    }

    fn fields_len(&self) -> usize {
        self.field_names.len()
    }

    fn field_at(&self, index: usize) -> Option<&dyn Reflect> {
        let key = self.field_names.get(index)?;
        Some(self.fields.get(key)?)
    }

    fn name_at(&self, index: usize) -> Option<&str> {
        self.field_names.get(index).map(|s| &**s)
    }

    fn field_at_mut(&mut self, index: usize) -> Option<&mut dyn Reflect> {
        let key = self.field_names.get(index)?;
        Some(self.fields.get_mut(key)?)
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
        let mut out = Self::default();
        for (name, value) in iter {
            out.set_field(name, value.to_value());
        }
        out
    }
}

#[derive(Debug)]
pub struct Iter<'a> {
    struct_: &'a dyn Struct,
    index: usize,
}

impl<'a> Iter<'a> {
    pub fn new(struct_: &'a dyn Struct) -> Self {
        Self { struct_, index: 0 }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = (&'a str, &'a dyn Reflect);

    fn next(&mut self) -> Option<Self::Item> {
        let name = self.struct_.name_at(self.index)?;
        let value = self.struct_.field_at(self.index)?;
        self.index += 1;
        Some((name, value))
    }
}

impl<'a> ExactSizeIterator for Iter<'a> {
    fn len(&self) -> usize {
        self.struct_.fields_len()
    }
}

impl<'a> FusedIterator for Iter<'a> {}
