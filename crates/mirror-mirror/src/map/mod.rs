use core::fmt;

use alloc::boxed::Box;

use crate::iter::PairIterMut;
use crate::Reflect;

mod unordered_map;
pub use unordered_map::UnorderedMap;

mod ordered_map;
pub use ordered_map::OrderedMap;

/// A reflected key-to-value map type.
///
/// Maps are guaranteed to not have duplicate entries for the same key, but there is
/// *not* a guaranteed of a stable ordering of the `(key, value)` elements in the map.
///
/// This is implemented for the std [`BTreeMap`], [`HashMap`], as well as for our own
/// [`UnorderedMap`], [`OrderedMap`] and [`Value`]s which are maps.
///
/// [`BTreeMap`]: alloc::collections::BTreeMap
/// [`HashMap`]: std::collections::HashMap
/// [`Value`]: crate::Value
pub trait Map: Reflect {
    fn get(&self, key: &dyn Reflect) -> Option<&dyn Reflect>;

    fn get_mut(&mut self, key: &dyn Reflect) -> Option<&mut dyn Reflect>;

    fn insert(&mut self, key: &dyn Reflect, value: &dyn Reflect) -> Option<Box<dyn Reflect>>;

    fn remove(&mut self, key: &dyn Reflect) -> Option<Box<dyn Reflect>>;

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool;

    /// Get an iterator over the `(k, v)` element pairs in the map. Note that the iteration order is *not*
    /// guaranteed to be stable.
    fn iter(&self) -> Iter<'_>;

    /// Get an iterator over the `(k, v)` element pairs in the map with mutable values. Note that the iteration order is *not*
    /// guaranteed to be stable.
    fn iter_mut(&mut self) -> PairIterMut<'_, dyn Reflect>;
}

impl fmt::Debug for dyn Map {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_reflect().debug(f)
    }
}

pub type Iter<'a> = Box<dyn Iterator<Item = (&'a dyn Reflect, &'a dyn Reflect)> + 'a>;
