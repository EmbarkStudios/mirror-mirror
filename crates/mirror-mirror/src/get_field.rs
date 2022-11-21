use crate::{Enum, Reflect, Struct, Tuple};

use self::private::*;

pub trait GetField {
    fn get_field<T>(&self, name: impl AsKey) -> Option<&T>
    where
        T: Reflect;

    fn get_field_mut<T>(&mut self, name: impl AsKey) -> Option<&mut T>
    where
        T: Reflect;
}

mod private {
    #![allow(unreachable_pub)]
    pub trait Sealed {}
    impl Sealed for &str {}
    impl Sealed for usize {}

    pub enum Key<'a> {
        Str(&'a str),
        Usize(usize),
    }
}

pub trait AsKey: Sealed {
    fn as_key(&self) -> Key<'_>;
}

impl AsKey for &str {
    fn as_key(&self) -> Key<'_> {
        Key::Str(self)
    }
}

impl AsKey for usize {
    fn as_key(&self) -> Key<'_> {
        Key::Usize(*self)
    }
}

impl<K> GetField for K
where
    K: Reflect,
{
    fn get_field<T>(&self, name: impl AsKey) -> Option<&T>
    where
        T: Reflect,
    {
        if let Some(struct_) = self.as_struct() {
            struct_.get_field(name)
        } else if let Some(enum_) = self.as_enum() {
            enum_.get_field(name)
        } else if let Some(tuple) = self.as_tuple() {
            tuple.get_field(name)
        } else {
            None
        }
    }

    fn get_field_mut<T>(&mut self, name: impl AsKey) -> Option<&mut T>
    where
        T: Reflect,
    {
        if self.as_struct_mut().is_some() {
            self.as_struct_mut().unwrap().get_field_mut(name)
        } else if self.as_enum_mut().is_some() {
            self.as_enum_mut().unwrap().get_field_mut(name)
        } else if self.as_tuple_mut().is_some() {
            self.as_tuple_mut().unwrap().get_field_mut(name)
        } else {
            None
        }
    }
}

impl GetField for dyn Struct {
    fn get_field<T>(&self, name: impl AsKey) -> Option<&T>
    where
        T: Reflect,
    {
        match name.as_key() {
            Key::Str(name) => self.field(name)?.downcast_ref(),
            Key::Usize(_) => None,
        }
    }

    fn get_field_mut<T>(&mut self, name: impl AsKey) -> Option<&mut T>
    where
        T: Reflect,
    {
        match name.as_key() {
            Key::Str(name) => self.field_mut(name)?.downcast_mut(),
            Key::Usize(_) => None,
        }
    }
}

impl GetField for dyn Enum {
    fn get_field<T>(&self, name: impl AsKey) -> Option<&T>
    where
        T: Reflect,
    {
        match name.as_key() {
            Key::Str(name) => self.field(name)?.downcast_ref(),
            Key::Usize(_) => None,
        }
    }

    fn get_field_mut<T>(&mut self, name: impl AsKey) -> Option<&mut T>
    where
        T: Reflect,
    {
        match name.as_key() {
            Key::Str(name) => self.field_mut(name)?.downcast_mut(),
            Key::Usize(_) => None,
        }
    }
}

impl GetField for dyn Tuple {
    fn get_field<T>(&self, name: impl AsKey) -> Option<&T>
    where
        T: Reflect,
    {
        match name.as_key() {
            Key::Str(_) => None,
            Key::Usize(index) => self.element(index)?.downcast_ref(),
        }
    }

    fn get_field_mut<T>(&mut self, name: impl AsKey) -> Option<&mut T>
    where
        T: Reflect,
    {
        match name.as_key() {
            Key::Str(_) => None,
            Key::Usize(index) => self.element_mut(index)?.downcast_mut(),
        }
    }
}
