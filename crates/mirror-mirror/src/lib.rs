#![deny(unreachable_pub)]
#![warn(clippy::todo)]

use std::{
    any::{Any, TypeId},
    fmt,
};

// TODO(david):
// - tuple structs
// - unit structs
// - unit enum variants
// - tuple enum variant
//     - option
//     - result
// - type info
// - vec
// - hash map/set
// - btree map/set
// - modifying
// - Box<T> where T: Reflect
// - impl FromIterator for StructValue

pub mod enum_;
pub mod struct_;
pub mod tuple;

mod get_field;
mod iter;
mod value;

#[cfg(test)]
mod tests;

#[doc(inline)]
pub use self::{
    enum_::{Enum, EnumValue},
    get_field::*,
    iter::*,
    struct_::{Struct, StructValue},
    tuple::{Tuple, TupleValue},
    value::*,
};

pub use mirror_mirror_macros::*;

pub trait Reflect: Any + Send + 'static {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn as_reflect(&self) -> &dyn Reflect;
    fn as_reflect_mut(&mut self) -> &mut dyn Reflect;

    fn as_tuple(&self) -> Option<&dyn Tuple>;
    fn as_tuple_mut(&mut self) -> Option<&mut dyn Tuple>;

    fn as_struct(&self) -> Option<&dyn Struct>;
    fn as_struct_mut(&mut self) -> Option<&mut dyn Struct>;

    fn as_enum(&self) -> Option<&dyn Enum>;
    fn as_enum_mut(&mut self) -> Option<&mut dyn Enum>;

    fn patch(&mut self, value: &dyn Reflect);

    fn to_value(&self) -> Value;

    fn clone_reflect(&self) -> Box<dyn Reflect>;

    fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;

    fn type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    fn type_name(&self) -> &str {
        std::any::type_name::<Self>()
    }
}

impl dyn Reflect {
    pub fn downcast_ref<T>(&self) -> Option<&T>
    where
        T: Reflect,
    {
        self.as_any().downcast_ref::<T>()
    }

    pub fn downcast_mut<T>(&mut self) -> Option<&mut T>
    where
        T: Reflect,
    {
        self.as_any_mut().downcast_mut::<T>()
    }
}

impl fmt::Debug for dyn Reflect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.debug(f)
    }
}

impl Reflect for Box<dyn Reflect> {
    fn as_any(&self) -> &dyn Any {
        <dyn Reflect as Reflect>::as_any(&**self)
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        <dyn Reflect as Reflect>::as_any_mut(&mut **self)
    }

    fn as_reflect(&self) -> &dyn Reflect {
        <dyn Reflect as Reflect>::as_reflect(&**self)
    }

    fn as_reflect_mut(&mut self) -> &mut dyn Reflect {
        <dyn Reflect as Reflect>::as_reflect_mut(&mut **self)
    }

    fn as_tuple(&self) -> Option<&dyn Tuple> {
        <dyn Reflect as Reflect>::as_tuple(&**self)
    }

    fn as_tuple_mut(&mut self) -> Option<&mut dyn Tuple> {
        <dyn Reflect as Reflect>::as_tuple_mut(&mut **self)
    }

    fn as_struct(&self) -> Option<&dyn Struct> {
        <dyn Reflect as Reflect>::as_struct(&**self)
    }

    fn as_struct_mut(&mut self) -> Option<&mut dyn Struct> {
        <dyn Reflect as Reflect>::as_struct_mut(&mut **self)
    }

    fn as_enum(&self) -> Option<&dyn Enum> {
        <dyn Reflect as Reflect>::as_enum(&**self)
    }

    fn as_enum_mut(&mut self) -> Option<&mut dyn Enum> {
        <dyn Reflect as Reflect>::as_enum_mut(&mut **self)
    }

    fn patch(&mut self, value: &dyn Reflect) {
        <dyn Reflect as Reflect>::patch(&mut **self, value)
    }

    fn to_value(&self) -> Value {
        <dyn Reflect as Reflect>::to_value(&**self)
    }

    fn clone_reflect(&self) -> Box<dyn Reflect> {
        <dyn Reflect as Reflect>::clone_reflect(&**self)
    }

    fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <dyn Reflect as Reflect>::debug(&**self, f)
    }
}

macro_rules! impl_for_core_types {
    ($($ty:ident)*) => {
        $(
            impl Reflect for $ty {
                fn as_any(&self) -> &dyn Any {
                    self
                }

                fn as_any_mut(&mut self) -> &mut dyn Any {
                    self
                }

                fn as_reflect(&self) -> &dyn Reflect {
                    self
                }

                fn as_reflect_mut(&mut self) -> &mut dyn Reflect {
                    self
                }

                fn patch(&mut self, value: &dyn Reflect) {
                    if let Some(value) = value.as_any().downcast_ref::<Self>() {
                        *self = value.clone();
                    }
                }

                fn clone_reflect(&self) -> Box<dyn Reflect> {
                    Box::new(self.clone())
                }

                fn to_value(&self) -> Value {
                    Value::from(self.to_owned())
                }

                fn as_tuple(&self) -> Option<&dyn Tuple> {
                    None
                }

                fn as_tuple_mut(&mut self) -> Option<&mut dyn Tuple> {
                    None
                }

                fn as_struct(&self) -> Option<&dyn Struct> {
                    None
                }

                fn as_struct_mut(&mut self) -> Option<&mut dyn Struct> {
                    None
                }

                fn as_enum(&self) -> Option<&dyn Enum> {
                    None
                }

                fn as_enum_mut(&mut self) -> Option<&mut dyn Enum> {
                    None
                }

                fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                        if f.alternate() {
                            write!(f, "{:#?}", self)
                        } else {
                            write!(f, "{:?}", self)
                        }
                }
            }

            impl FromReflect for $ty {
                fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
                    Some(reflect.downcast_ref::<$ty>()?.clone())
                }
            }
        )*
    };
}

impl_for_core_types! {
    usize u8 u16 u32 u64 u128
    i8 i16 i32 i64 i128
    f32 f64
    bool char String
}

pub trait FromReflect: Reflect + Sized {
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self>;
}

/// Private. Used by macros
#[doc(hidden)]
pub mod __private {
    pub use crate::FromReflect;
    pub use crate::Reflect;
    pub use crate::Value;
    pub use std::any::Any;
    pub use std::fmt;
}
