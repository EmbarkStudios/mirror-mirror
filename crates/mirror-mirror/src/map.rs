use crate::iter::PairIter;
use crate::iter::PairIterMut;
use crate::FromReflect;
use crate::Reflect;
use crate::ReflectMut;
use crate::ReflectRef;
use crate::TypeInfo;
use crate::Typed;
use crate::Value;
use std::any::Any;
use std::collections::BTreeMap;
use std::fmt;

pub trait Map: Reflect {
    fn get(&self, key: &dyn Reflect) -> Option<&dyn Reflect>;

    fn get_mut(&mut self, key: &dyn Reflect) -> Option<&mut dyn Reflect>;

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool;

    fn iter(&self) -> PairIter<'_, dyn Reflect>;

    fn iter_mut(&mut self) -> PairIterMut<'_, dyn Reflect>;
}

impl fmt::Debug for dyn Map {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_reflect().debug(f)
    }
}

impl<K, V> Map for BTreeMap<K, V>
where
    K: FromReflect + Ord,
    V: FromReflect,
{
    fn get(&self, key: &dyn Reflect) -> Option<&dyn Reflect> {
        let key = key.downcast_ref::<K>()?;
        let value = self.get(key)?;
        Some(value.as_reflect())
    }

    fn get_mut(&mut self, key: &dyn Reflect) -> Option<&mut dyn Reflect> {
        let key = key.downcast_ref::<K>()?;
        let value = self.get_mut(key)?;
        Some(value.as_reflect_mut())
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    fn iter(&self) -> PairIter<'_, dyn Reflect> {
        let iter = self
            .iter()
            .map(|(key, value)| (key.as_reflect(), value.as_reflect()));
        PairIter::new(iter)
    }

    fn iter_mut(&mut self) -> PairIterMut<'_, dyn Reflect> {
        let iter = self
            .iter_mut()
            .map(|(key, value)| (key.as_reflect(), value.as_reflect_mut()));
        PairIterMut::new(iter)
    }
}

impl<K, V> Reflect for BTreeMap<K, V>
where
    K: FromReflect + Ord,
    V: FromReflect,
{
    fn type_info(&self) -> TypeInfo {
        <Self as Typed>::type_info()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn as_reflect(&self) -> &dyn Reflect {
        self
    }

    fn as_reflect_mut(&mut self) -> &mut dyn Reflect {
        self
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

impl<K, V> FromReflect for BTreeMap<K, V>
where
    K: FromReflect + Ord,
    V: FromReflect,
{
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
        let map = reflect.as_reflect().as_map()?;
        let mut out = BTreeMap::new();
        for (key, value) in map.iter() {
            out.insert(K::from_reflect(key)?, V::from_reflect(value)?);
        }
        Some(out)
    }
}
