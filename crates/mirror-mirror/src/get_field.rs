use crate::{Enum, List, Reflect, ReflectMut, ReflectRef, Struct, Tuple, TupleStruct};

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
        match self.reflect_ref() {
            ReflectRef::Struct(inner) => inner.get_field(name),
            ReflectRef::TupleStruct(inner) => inner.get_field(name),
            ReflectRef::Tuple(inner) => inner.get_field(name),
            ReflectRef::Enum(inner) => inner.get_field(name),
            ReflectRef::List(inner) => inner.get_field(name),
            ReflectRef::Scalar(_) => None,
        }
    }

    fn get_field_mut<T>(&mut self, name: impl AsKey) -> Option<&mut T>
    where
        T: Reflect,
    {
        match self.reflect_mut() {
            ReflectMut::Struct(inner) => inner.get_field_mut(name),
            ReflectMut::TupleStruct(inner) => inner.get_field_mut(name),
            ReflectMut::Tuple(inner) => inner.get_field_mut(name),
            ReflectMut::Enum(inner) => inner.get_field_mut(name),
            ReflectMut::List(inner) => inner.get_field_mut(name),
            ReflectMut::Scalar(_) => None,
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

impl GetField for dyn TupleStruct {
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

impl GetField for dyn Enum {
    fn get_field<T>(&self, name: impl AsKey) -> Option<&T>
    where
        T: Reflect,
    {
        match name.as_key() {
            Key::Str(name) => self.field(name)?.downcast_ref(),
            Key::Usize(index) => self.element(index)?.downcast_ref(),
        }
    }

    fn get_field_mut<T>(&mut self, name: impl AsKey) -> Option<&mut T>
    where
        T: Reflect,
    {
        match name.as_key() {
            Key::Str(name) => self.field_mut(name)?.downcast_mut(),
            Key::Usize(index) => self.element_mut(index)?.downcast_mut(),
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

impl GetField for dyn List {
    fn get_field<T>(&self, name: impl AsKey) -> Option<&T>
    where
        T: Reflect,
    {
        match name.as_key() {
            Key::Str(_) => None,
            Key::Usize(index) => self.get(index)?.downcast_ref(),
        }
    }

    fn get_field_mut<T>(&mut self, name: impl AsKey) -> Option<&mut T>
    where
        T: Reflect,
    {
        match name.as_key() {
            Key::Str(_) => None,
            Key::Usize(index) => self.get_mut(index)?.downcast_mut(),
        }
    }
}
