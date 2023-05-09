use core::any::Any;
use core::fmt;

use kollect::LinearMap;

use crate::iter::PairIterMut;
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

impl<K, V> Reflect for LinearMap<K, V>
where
    K: FromReflect + DescribeType + Eq,
    V: FromReflect + DescribeType,
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

impl<K, V> FromReflect for LinearMap<K, V>
where
    K: FromReflect + DescribeType + Eq,
    V: FromReflect + DescribeType,
{
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
        let map = reflect.as_map()?;
        let len = map.len();
        let mut out = LinearMap::with_capacity(len);
        for (key, value) in map.iter() {
            out.insert(K::from_reflect(key)?, V::from_reflect(value)?);
        }
        Some(out)
    }
}

impl<K, V> From<LinearMap<K, V>> for Value
where
    K: Reflect + Eq,
    V: Reflect,
{
    fn from(map: LinearMap<K, V>) -> Self {
        let map = map
            .into_iter()
            .map(|(key, value)| (key.to_value(), value.to_value()))
            .collect();
        Value::Map(map)
    }
}

impl<K, V> Map for LinearMap<K, V>
where
    K: FromReflect + DescribeType + Eq,
    V: FromReflect + DescribeType,
{
    fn get(&self, key: &dyn Reflect) -> Option<&dyn Reflect> {
        let key = K::from_reflect(key)?;
        let value = self.get(&key)?;
        Some(value.as_reflect())
    }

    fn get_mut(&mut self, key: &dyn Reflect) -> Option<&mut dyn Reflect> {
        let key = K::from_reflect(key)?;
        let value = self.get_mut(&key)?;
        Some(value.as_reflect_mut())
    }

    fn insert(&mut self, key: &dyn Reflect, value: &dyn Reflect) -> Option<Box<dyn Reflect>> {
        let key = K::from_reflect(key)?;
        let value = V::from_reflect(value)?;
        let previous = self.insert(key, value)?;
        Some(Box::new(previous))
    }

    fn remove(&mut self, key: &dyn Reflect) -> Option<Box<dyn Reflect>> {
        let key = K::from_reflect(key)?;
        let previous = self.remove(&key)?;
        Some(Box::new(previous))
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    fn iter(&self) -> crate::map::Iter<'_> {
        let iter = self
            .iter()
            .map(|(key, value)| (key.as_reflect(), value.as_reflect()));
        Box::new(iter)
    }

    fn iter_mut(&mut self) -> PairIterMut<'_, dyn Reflect> {
        let iter = self
            .iter_mut()
            .map(|(key, value)| (key.as_reflect(), value.as_reflect_mut()));
        Box::new(iter)
    }
}

impl<K, V> DescribeType for LinearMap<K, V>
where
    K: DescribeType,
    V: DescribeType,
{
    fn build(graph: &mut TypeGraph) -> NodeId {
        graph.get_or_build_node_with::<Self, _>(|graph| MapNode::new::<Self, K, V>(graph))
    }
}
