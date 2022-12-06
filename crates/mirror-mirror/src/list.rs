use alloc::boxed::Box;
use core::fmt;

use crate::array::Array;
use crate::Reflect;

pub trait List: Array {
    fn push(&mut self, value: &dyn Reflect);

    fn pop(&mut self) -> Option<Box<dyn Reflect>>;

    fn try_remove(&mut self, index: usize) -> Option<Box<dyn Reflect>>;
}

impl fmt::Debug for dyn List {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_reflect().debug(f)
    }
}
