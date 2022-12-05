use crate::iter::PairIter;
use crate::iter::PairIterMut;
use crate::Reflect;
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
