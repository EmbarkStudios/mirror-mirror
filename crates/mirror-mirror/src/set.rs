use crate::Reflect;

use core::fmt;

pub trait Set: Reflect {
    fn len(&self) -> usize;

    fn is_empty(&self) -> bool;

    fn try_insert(&mut self, element: &dyn Reflect) -> Result<bool, SetError>;

    fn try_remove(&mut self, element: &dyn Reflect) -> Result<bool, SetError>;

    fn try_contains(&mut self, element: &dyn Reflect) -> Result<bool, SetError>;

    fn iter(&self) -> Iter<'_>;
}

impl fmt::Debug for dyn Set {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_reflect().debug(f)
    }
}

pub type Iter<'a> = Box<dyn Iterator<Item = &'a dyn Reflect> + 'a>;

/// A method on a reflected set failed.
#[non_exhaustive]
#[derive(Debug)]
pub struct SetError;

impl core::fmt::Display for SetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "failed to parse element")
    }
}

impl std::error::Error for SetError {}
