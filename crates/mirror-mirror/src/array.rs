use crate::iter::ValueIter;
use crate::iter::ValueIterMut;
use crate::Reflect;
use std::fmt;

pub trait Array: Reflect {
    fn get(&self, index: usize) -> Option<&dyn Reflect>;

    fn get_mut(&mut self, index: usize) -> Option<&mut dyn Reflect>;

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool;

    fn iter(&self) -> ValueIter<'_>;

    fn iter_mut(&mut self) -> ValueIterMut<'_>;
}

impl fmt::Debug for dyn Array {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_reflect().debug(f)
    }
}
