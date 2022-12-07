use alloc::boxed::Box;

use crate::Reflect;

// Its not possible to implement this without boxing, because rust cannot prove that the borrows
// from `next` don't overlap. That requires `LendingIterator`
//
// Its a type alias to make it clear that it allocates
pub type ValueIterMut<'a> = Box<dyn Iterator<Item = &'a mut dyn Reflect> + 'a>;

// Its not possible to implement this without boxing, because rust cannot prove that the borrows
// from `next` don't overlap. That requires `LendingIterator`
//
// Its a type alias to make it clear that it allocates
pub type PairIterMut<'a, T = str> = Box<dyn Iterator<Item = (&'a T, &'a mut dyn Reflect)> + 'a>;
