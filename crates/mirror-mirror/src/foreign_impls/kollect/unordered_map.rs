use core::any::Any;
use core::fmt;
use core::hash::BuildHasher;
use core::hash::Hash;

use kollect::UnorderedMap;

use crate::iter::PairIterMut;
use crate::map::MapError;
use crate::type_info::graph::MapNode;
use crate::type_info::graph::NodeId;
use crate::type_info::graph::TypeGraph;
use crate::DescribeType;
use crate::FromReflect;
use crate::Map;
use crate::Reflect;
use crate::ReflectMut;
use crate::ReflectOwned;
use crate::ReflectRef;
use crate::Value;

impl<K, V, S> Reflect for UnorderedMap<K, V, S>
where
    K: FromReflect + DescribeType + Eq + Hash,
    V: FromReflect + DescribeType,
    S: Default + BuildHasher + Send + 'static,
{
    trivial_reflect_methods!();

    fn reflect_owned(self: Box<Self>) -> ReflectOwned {
        ReflectOwned::Map(self)
    }

    fn reflect_ref(&self) -> ReflectRef<'_> {
        ReflectRef::Map(self)
    }

    fn reflect_mut(&mut self) -> ReflectMut<'_> {
        ReflectMut::Map(self)
    }

    fn patch(&mut self, value: &dyn Reflect) {
        if let Some(map) = value.reflect_ref().as_map() {
            for (key, new_value) in map.iter() {
                if let Some(value) = Map::get_mut(self, key) {
                    value.patch(new_value);
                }
            }
        }
    }

    fn to_value(&self) -> Value {
        let data = self
            .iter()
            .map(|(key, value)| (key.to_value(), value.to_value()))
            .collect();
        Value::Map(data)
    }

    fn clone_reflect(&self) -> Box<dyn Reflect> {
        let value = self.to_value();
        Box::new(Self::from_reflect(&value).unwrap())
    }

    fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(Map::iter(self)).finish()
    }
}

impl<K, V, S> FromReflect for UnorderedMap<K, V, S>
where
    K: FromReflect + DescribeType + Eq + Hash,
    V: FromReflect + DescribeType,
    S: Default + BuildHasher + Send + 'static,
{
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
        let map = reflect.as_map()?;
        let len = map.len();
        let mut out = UnorderedMap::with_capacity_and_hasher(len, S::default());
        for (key, value) in map.iter() {
            out.insert(K::from_reflect(key)?, V::from_reflect(value)?);
        }
        Some(out)
    }
}

impl<K, V> From<UnorderedMap<K, V>> for Value
where
    K: Reflect,
    V: Reflect,
{
    fn from(map: UnorderedMap<K, V>) -> Self {
        let map = map
            .into_iter()
            .map(|(key, value)| (key.to_value(), value.to_value()))
            .collect();
        Value::Map(map)
    }
}

impl<K, V, S> Map for UnorderedMap<K, V, S>
where
    K: FromReflect + DescribeType + Hash + Eq,
    V: FromReflect + DescribeType,
    S: Default + BuildHasher + Send + 'static,
{
    map_methods!();
}

impl<K, V, S> DescribeType for UnorderedMap<K, V, S>
where
    K: DescribeType,
    V: DescribeType,
    S: 'static,
{
    fn build(graph: &mut TypeGraph) -> NodeId {
        graph.get_or_build_node_with::<Self, _>(|graph| MapNode::new::<Self, K, V>(graph))
    }
}