use crate::{Enum, Reflect, Struct};

pub trait GetField {
    fn get_field<T>(&self, name: &str) -> Option<&T>
    where
        T: Reflect;

    fn get_field_mut<T>(&mut self, name: &str) -> Option<&mut T>
    where
        T: Reflect;
}

impl<K> GetField for K
where
    K: Reflect,
{
    fn get_field<T>(&self, name: &str) -> Option<&T>
    where
        T: Reflect,
    {
        if let Some(struct_) = self.as_struct() {
            struct_.get_field(name)
        } else if let Some(enum_) = self.as_enum() {
            enum_.get_field(name)
        } else {
            None
        }
    }

    fn get_field_mut<T>(&mut self, name: &str) -> Option<&mut T>
    where
        T: Reflect,
    {
        if self.as_struct_mut().is_some() {
            self.as_struct_mut().unwrap().get_field_mut(name)
        } else if self.as_enum_mut().is_some() {
            self.as_enum_mut().unwrap().get_field_mut(name)
        } else {
            None
        }
    }
}

impl GetField for dyn Struct {
    fn get_field<T>(&self, name: &str) -> Option<&T>
    where
        T: Reflect,
    {
        self.field(name)?.downcast_ref()
    }

    fn get_field_mut<T>(&mut self, name: &str) -> Option<&mut T>
    where
        T: Reflect,
    {
        self.field_mut(name)?.downcast_mut()
    }
}

impl GetField for dyn Enum {
    fn get_field<T>(&self, name: &str) -> Option<&T>
    where
        T: Reflect,
    {
        self.variant().field(name)?.downcast_ref()
    }

    fn get_field_mut<'a, T>(&'a mut self, name: &str) -> Option<&'a mut T>
    where
        T: Reflect,
    {
        self.variant_mut().into_field_mut(name)?.downcast_mut()
    }
}
