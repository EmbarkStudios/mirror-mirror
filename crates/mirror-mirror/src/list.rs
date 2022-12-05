use crate::array::Array;
use crate::Reflect;
use std::fmt;

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
