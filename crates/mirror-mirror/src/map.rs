use core::fmt;

use alloc::boxed::Box;

use crate::iter::PairIterMut;
use crate::{FromReflect, Reflect};

/// A reflected key-to-value map type.
///
/// Maps are guaranteed to not have duplicate entries for the same key, but there is
/// *not* a guaranteed of a stable ordering of the `(key, value)` elements in the map.
/// However, for underlying map types that have an ordering, that ordering can be assumed
/// to be respected.
///
/// This is implemented for the std [`BTreeMap`], [`HashMap`], as well as for our own
/// [`UnorderedMap`], [`OrderedMap`] and [`Value`]s which are maps.
///
/// [`BTreeMap`]: alloc::collections::BTreeMap
/// [`HashMap`]: std::collections::HashMap
/// [`Value`]: crate::Value
/// [`OrderedMap`]: kollect::OrderedMap
/// [`UnorderedMap`]: kollect::UnorderedMap
pub trait Map: Reflect {
    fn get(&self, key: &dyn Reflect) -> Option<&dyn Reflect>;

    fn get_mut(&mut self, key: &dyn Reflect) -> Option<&mut dyn Reflect>;

    /// Inserts a key-value pair into the map.
    ///
    /// If the map did not have this key present, `Ok(None)` is returned.
    ///
    /// If the key, or value, failed to be parsed with `FromReflect::from_reflect` the `Err(_)` is
    /// returned.
    fn try_insert<'a>(
        &mut self,
        key: &'a dyn Reflect,
        value: &'a dyn Reflect,
    ) -> Result<Option<Box<dyn Reflect>>, MapError>;

    /// Removes a key from the map, returning the value at the key if the key was previously in the
    /// map.
    ///
    /// If the key failed to be parsed with `FromReflect::from_reflect` the `Err(_)` is returned.
    fn try_remove(&mut self, key: &dyn Reflect) -> Result<Option<Box<dyn Reflect>>, MapError>;

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool;

    /// Get an iterator over the `(k, v)` element pairs in the map. Note that the iteration order
    /// is *not* guaranteed to be stable, though if the underlying implementor type does have a
    /// defined order then that can be assumed to be respected.
    fn iter(&self) -> Iter<'_>;

    /// Get an iterator over the `(k, v)` element pairs in the map with mutable values. Note that
    /// the iteration order is *not* guaranteed to be stable, though if the underlying implementor
    /// type does have a defined order then that can be assumed to be respected.
    fn iter_mut(&mut self) -> PairIterMut<'_, dyn Reflect>;
}

impl fmt::Debug for dyn Map {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_reflect().debug(f)
    }
}

pub type Iter<'a> = Box<dyn Iterator<Item = (&'a dyn Reflect, &'a dyn Reflect)> + 'a>;

/// A method on a reflected map failed.
#[derive(Debug)]
pub enum MapError {
    /// Parsing the key with `FromReflect::from_reflect` failed.
    KeyFromReflectFailed,
    /// Parsing the value with `FromReflect::from_reflect` failed.
    ValueFromReflectFailed,
}

impl core::fmt::Display for MapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MapError::KeyFromReflectFailed => write!(f, "failed to parse key"),
            MapError::ValueFromReflectFailed => write!(f, "failed to parse value"),
        }
    }
}

impl std::error::Error for MapError {}

pub(crate) fn key_value_from_reflect<K, V>(
    key: &dyn Reflect,
    value: &dyn Reflect,
) -> Result<(K, V), MapError>
where
    K: FromReflect,
    V: FromReflect,
{
    let k = K::from_reflect(key).ok_or(MapError::KeyFromReflectFailed)?;
    let v = V::from_reflect(value).ok_or(MapError::KeyFromReflectFailed)?;
    Ok((k, v))
}
