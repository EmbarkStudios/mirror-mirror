use core::fmt;
use core::iter::FusedIterator;

use crate::iter::ValueIterMut;
use crate::Reflect;

/// A reflected array type.
pub trait Array: Reflect {
    fn get(&self, index: usize) -> Option<&dyn Reflect>;

    fn get_mut(&mut self, index: usize) -> Option<&mut dyn Reflect>;

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool;

    fn iter(&self) -> Iter<'_>;

    fn iter_mut(&mut self) -> ValueIterMut<'_>;

    /// Swaps two elements in the array.
    ///
    /// # Panics
    ///
    /// Panics if `a` or `b` are out of bounds.
    fn swap(&mut self, a: usize, b: usize);
}

impl fmt::Debug for dyn Array {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_reflect().debug(f)
    }
}

#[derive(Debug)]
pub struct Iter<'a> {
    index: usize,
    array: &'a dyn Array,
}

impl<'a> Iter<'a> {
    pub fn new(array: &'a dyn Array) -> Self {
        Self { index: 0, array }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a dyn Reflect;

    fn next(&mut self) -> Option<Self::Item> {
        let value = self.array.get(self.index)?;
        self.index += 1;
        Some(value)
    }
}

impl<'a> ExactSizeIterator for Iter<'a> {
    fn len(&self) -> usize {
        self.array.len()
    }
}

impl<'a> FusedIterator for Iter<'a> {}
