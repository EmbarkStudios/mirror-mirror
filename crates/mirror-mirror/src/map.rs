use alloc::boxed::Box;
use core::fmt;

use crate::iter::PairIterMut;
use crate::Reflect;

/// A reflected map type.
///
/// Note this is only implemented for [`BTreeMap`] and _not_ [`HashMap`] due to technical
/// limitations.
///
/// [`BTreeMap`]: alloc::collections::BTreeMap
/// [`HashMap`]: std::collections::HashMap
// HashMap isn't supported because we need a `Value` variant for map values. The most obvious
// choice is `enum Value { Map(HashMap<Value, Value>) }`. However now `Value` is used as the key in
// a `HashMap` so it most implement `Hash + Eq` but it can't since it contains a `HashMap` which
// doesn't implement `Hash + Eq`, because there is no stable iteration order.
pub trait Map: Reflect {
    fn get(&self, key: &dyn Reflect) -> Option<&dyn Reflect>;

    fn get_mut(&mut self, key: &dyn Reflect) -> Option<&mut dyn Reflect>;

    fn insert(&mut self, key: &dyn Reflect, value: &dyn Reflect) -> Option<Box<dyn Reflect>>;

    fn remove(&mut self, key: &dyn Reflect) -> Option<Box<dyn Reflect>>;

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool;

    fn iter(&self) -> Iter<'_>;

    fn iter_mut(&mut self) -> PairIterMut<'_, dyn Reflect>;
}

impl fmt::Debug for dyn Map {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_reflect().debug(f)
    }
}

pub type Iter<'a> = Box<dyn Iterator<Item = (&'a dyn Reflect, &'a dyn Reflect)> + 'a>;
