use alloc::boxed::Box;
use core::fmt;

use crate::iter::PairIter;
use crate::iter::PairIterMut;
use crate::Reflect;

pub trait Map: Reflect {
    fn get(&self, key: &dyn Reflect) -> Option<&dyn Reflect>;

    fn get_mut(&mut self, key: &dyn Reflect) -> Option<&mut dyn Reflect>;

    fn insert(&mut self, key: &dyn Reflect, value: &dyn Reflect) -> Option<Box<dyn Reflect>>;

    fn remove(&mut self, key: &dyn Reflect) -> Option<Box<dyn Reflect>>;

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
