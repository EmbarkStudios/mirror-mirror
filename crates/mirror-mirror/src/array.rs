use core::fmt;

use crate::iter::ValueIterMut;
use crate::Reflect;

pub trait Array: Reflect {
    fn get(&self, index: usize) -> Option<&dyn Reflect>;

    fn get_mut(&mut self, index: usize) -> Option<&mut dyn Reflect>;

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool;

    fn iter(&self) -> Iter<'_>;

    fn iter_mut(&mut self) -> ValueIterMut<'_>;
}

impl fmt::Debug for dyn Array {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_reflect().debug(f)
    }
}

pub struct Iter<'a> {
    index: usize,
    reflect: &'a dyn Array,
}

impl<'a> Iter<'a> {
    pub fn new(reflect: &'a dyn Array) -> Self {
        Self { index: 0, reflect }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a dyn Reflect;

    fn next(&mut self) -> Option<Self::Item> {
        let value = self.reflect.get(self.index)?;
        self.index += 1;
        Some(value)
    }
}
