use core::any::Any;
use core::fmt;
use core::hash::Hash;
use std::hash::BuildHasher;

use kollect::OrderedSet;

use crate::{
    set::{Iter, SetError},
    type_info::graph::{NodeId, SetNode, TypeGraph},
    DescribeType, FromReflect, Reflect, ReflectMut, ReflectOwned, ReflectRef, Set, Value,
};

impl<V, S> Reflect for OrderedSet<V, S>
where
    V: FromReflect + DescribeType + Hash + Eq,
    S: Default + BuildHasher + Send + 'static,
{
    trivial_reflect_methods!();

    fn reflect_owned(self: Box<Self>) -> ReflectOwned {
        ReflectOwned::Set(self)
    }

    fn reflect_ref(&self) -> ReflectRef<'_> {
        ReflectRef::Set(self)
    }

    fn reflect_mut(&mut self) -> ReflectMut<'_> {
        ReflectMut::Set(self)
    }

    /// Performs a union.
    fn patch(&mut self, value: &dyn Reflect) {
        if let Some(set) = value.as_set() {
            for element in set.iter() {
                _ = self.try_insert(element);
            }
        }
    }

    fn to_value(&self) -> Value {
        let data = self.iter().map(Reflect::to_value).collect();
        Value::Set(data)
    }

    fn clone_reflect(&self) -> Box<dyn Reflect> {
        let value = self.to_value();
        Box::new(Self::from_reflect(&value).unwrap())
    }

    fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(Set::iter(self)).finish()
    }
}

impl<V, S> FromReflect for OrderedSet<V, S>
where
    V: FromReflect + DescribeType + Eq + Hash,
    S: Default + BuildHasher + Send + 'static,
{
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
        let set = reflect.as_set()?;
        let len = set.len();
        let mut out = OrderedSet::with_capacity_and_hasher(len, S::default());
        for element in set.iter() {
            out.insert(V::from_reflect(element)?);
        }
        Some(out)
    }
}

impl<V> From<OrderedSet<V>> for Value
where
    V: Reflect,
{
    fn from(set: OrderedSet<V>) -> Self {
        let set = set.into_iter().map(|element| element.to_value()).collect();
        Value::Set(set)
    }
}

impl<V, S> Set for OrderedSet<V, S>
where
    V: FromReflect + DescribeType + Eq + Hash,
    S: Default + BuildHasher + Send + 'static,
{
    set_methods!();
}

impl<V, S> DescribeType for OrderedSet<V, S>
where
    V: DescribeType,
    S: 'static,
{
    fn build(graph: &mut TypeGraph) -> NodeId {
        graph.get_or_build_node_with::<Self, _>(|graph| SetNode::new::<Self, V>(graph))
    }
}
