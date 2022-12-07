#![cfg_attr(not(feature = "std"), no_std)]
#![warn(
    clippy::all,
    clippy::dbg_macro,
    clippy::todo,
    clippy::empty_enum,
    clippy::enum_glob_use,
    clippy::mem_forget,
    clippy::unused_self,
    clippy::filter_map_next,
    clippy::needless_continue,
    clippy::needless_borrow,
    clippy::match_wildcard_for_single_variants,
    clippy::if_let_mutex,
    clippy::mismatched_target_os,
    clippy::await_holding_lock,
    clippy::match_on_vec_items,
    clippy::imprecise_flops,
    clippy::suboptimal_flops,
    clippy::lossy_float_literal,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::fn_params_excessive_bools,
    clippy::exit,
    clippy::inefficient_to_string,
    clippy::linkedlist,
    clippy::macro_use_imports,
    clippy::option_option,
    clippy::verbose_file_reads,
    clippy::unnested_or_patterns,
    clippy::str_to_string,
    rust_2018_idioms,
    future_incompatible,
    nonstandard_style,
    // missing_debug_implementations,
    // missing_docs
)]
#![deny(unreachable_pub, private_in_public)]
#![allow(
    elided_lifetimes_in_paths,
    clippy::type_complexity,
    // because speedy
    clippy::not_unsafe_ptr_arg_deref,
)]
#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![cfg_attr(test, allow(clippy::float_cmp))]

extern crate alloc;

use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::string::String;
use core::any::Any;
use core::any::TypeId;
use core::fmt;

pub mod array;
pub mod enum_;
pub mod get_field;
pub mod iter;
pub mod key_path;
pub mod list;
pub mod map;
pub mod struct_;
pub mod tuple;
pub mod tuple_struct;
pub mod type_info;
pub mod value;

mod std_impls;

#[cfg(feature = "std")]
#[cfg(test)]
mod tests;

#[doc(inline)]
pub use mirror_mirror_macros::*;

#[doc(inline)]
pub use self::array::Array;
#[doc(inline)]
pub use self::enum_::Enum;
#[doc(inline)]
pub use self::get_field::GetField;
#[doc(inline)]
pub use self::get_field::GetFieldMut;
#[doc(inline)]
pub use self::list::List;
#[doc(inline)]
pub use self::map::Map;
#[doc(inline)]
pub use self::struct_::Struct;
#[doc(inline)]
pub use self::tuple::Tuple;
#[doc(inline)]
pub use self::tuple_struct::TupleStruct;
#[doc(inline)]
pub use self::type_info::TypeInfoRoot;
#[doc(inline)]
pub use self::type_info::Typed;
#[doc(inline)]
pub use self::value::Value;

pub trait Reflect: Any + Send + 'static {
    fn type_info(&self) -> TypeInfoRoot;

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn as_reflect(&self) -> &dyn Reflect;

    fn as_reflect_mut(&mut self) -> &mut dyn Reflect;

    fn reflect_ref(&self) -> ReflectRef<'_>;

    fn reflect_mut(&mut self) -> ReflectMut<'_>;

    fn patch(&mut self, value: &dyn Reflect);

    fn to_value(&self) -> Value;

    fn clone_reflect(&self) -> Box<dyn Reflect>;

    fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;

    fn type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    fn type_name(&self) -> &str {
        core::any::type_name::<Self>()
    }

    fn as_tuple(&self) -> Option<&dyn Tuple> {
        self.reflect_ref().as_tuple()
    }

    fn as_tuple_mut(&mut self) -> Option<&mut dyn Tuple> {
        self.reflect_mut().as_tuple_mut()
    }

    fn as_struct(&self) -> Option<&dyn Struct> {
        self.reflect_ref().as_struct()
    }

    fn as_struct_mut(&mut self) -> Option<&mut dyn Struct> {
        self.reflect_mut().as_struct_mut()
    }

    fn as_tuple_struct(&self) -> Option<&dyn TupleStruct> {
        self.reflect_ref().as_tuple_struct()
    }

    fn as_tuple_struct_mut(&mut self) -> Option<&mut dyn TupleStruct> {
        self.reflect_mut().as_tuple_struct_mut()
    }

    fn as_enum(&self) -> Option<&dyn Enum> {
        self.reflect_ref().as_enum()
    }

    fn as_enum_mut(&mut self) -> Option<&mut dyn Enum> {
        self.reflect_mut().as_enum_mut()
    }

    fn as_list(&self) -> Option<&dyn List> {
        self.reflect_ref().as_list()
    }

    fn as_list_mut(&mut self) -> Option<&mut dyn List> {
        self.reflect_mut().as_list_mut()
    }

    fn as_array(&self) -> Option<&dyn Array> {
        self.reflect_ref().as_array()
    }

    fn as_array_mut(&mut self) -> Option<&mut dyn Array> {
        self.reflect_mut().as_array_mut()
    }

    fn as_map(&self) -> Option<&dyn Map> {
        self.reflect_ref().as_map()
    }

    fn as_map_mut(&mut self) -> Option<&mut dyn Map> {
        self.reflect_mut().as_map_mut()
    }

    fn as_scalar(&self) -> Option<ScalarRef<'_>> {
        self.reflect_ref().as_scalar()
    }

    fn as_scalar_mut(&mut self) -> Option<ScalarMut<'_>> {
        self.reflect_mut().as_scalar_mut()
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
    fn type_info(&self) -> TypeInfoRoot {
        <dyn Reflect as Reflect>::type_info(&**self)
    }

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

    fn reflect_ref(&self) -> ReflectRef<'_> {
        <dyn Reflect as Reflect>::reflect_ref(&**self)
    }

    fn reflect_mut(&mut self) -> ReflectMut<'_> {
        <dyn Reflect as Reflect>::reflect_mut(&mut **self)
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
                fn type_info(&self) -> TypeInfoRoot {
                    <Self as Typed>::type_info()
                }

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

                fn reflect_ref(&self) -> ReflectRef<'_> {
                    ReflectRef::Scalar(ScalarRef::from(*self))
                }

                fn reflect_mut(&mut self) -> ReflectMut<'_> {
                    ReflectMut::Scalar(ScalarMut::from(self))
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

            impl From<$ty> for ScalarRef<'static> {
                fn from(value: $ty) -> Self {
                    ScalarRef::$ty(value)
                }
            }

            impl<'a> From<&'a mut $ty> for ScalarMut<'a> {
                fn from(value: &'a mut $ty) -> Self {
                    ScalarMut::$ty(value)
                }
            }
        )*
    };
}

impl_for_core_types! {
    usize u8 u16 u32 u64 u128
    i8 i16 i32 i64 i128
    f32 f64
    bool char
}

impl Reflect for String {
    fn type_info(&self) -> TypeInfoRoot {
        <Self as Typed>::type_info()
    }

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

    fn reflect_ref(&self) -> ReflectRef<'_> {
        ReflectRef::Scalar(ScalarRef::from(self))
    }

    fn reflect_mut(&mut self) -> ReflectMut<'_> {
        ReflectMut::Scalar(ScalarMut::String(self))
    }

    fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "{:#?}", self)
        } else {
            write!(f, "{:?}", self)
        }
    }
}

impl FromReflect for String {
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
        Some(reflect.downcast_ref::<String>()?.clone())
    }
}

impl<'a> From<&'a String> for ScalarRef<'a> {
    fn from(value: &'a String) -> Self {
        ScalarRef::String(value.as_str())
    }
}

impl<'a> From<&'a mut String> for ScalarMut<'a> {
    fn from(value: &'a mut String) -> Self {
        ScalarMut::String(value)
    }
}

pub trait FromReflect: Reflect + Sized {
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self>;
}

#[derive(Debug, Copy, Clone)]
pub enum ReflectRef<'a> {
    Struct(&'a dyn Struct),
    TupleStruct(&'a dyn TupleStruct),
    Tuple(&'a dyn Tuple),
    Enum(&'a dyn Enum),
    Array(&'a dyn Array),
    List(&'a dyn List),
    Map(&'a dyn Map),
    Scalar(ScalarRef<'a>),
    /// Not all `Reflect` implementations allow access to the underlying value. This variant can be
    /// used for such types.
    Opaque(&'a dyn Reflect),
}

impl<'a> ReflectRef<'a> {
    pub fn as_tuple(self) -> Option<&'a dyn Tuple> {
        match self {
            Self::Tuple(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_struct(self) -> Option<&'a dyn Struct> {
        match self {
            Self::Struct(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_tuple_struct(self) -> Option<&'a dyn TupleStruct> {
        match self {
            Self::TupleStruct(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_enum(self) -> Option<&'a dyn Enum> {
        match self {
            Self::Enum(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_list(self) -> Option<&'a dyn List> {
        match self {
            Self::List(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_array(self) -> Option<&'a dyn Array> {
        match self {
            Self::Array(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_map(self) -> Option<&'a dyn Map> {
        match self {
            Self::Map(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_scalar(self) -> Option<ScalarRef<'a>> {
        match self {
            Self::Scalar(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_opaque(self) -> Option<&'a dyn Reflect> {
        match self {
            Self::Opaque(inner) => Some(inner),
            _ => None,
        }
    }
}

#[derive(Debug, Copy, Clone)]
#[allow(non_camel_case_types)]
pub enum ScalarRef<'a> {
    usize(usize),
    u8(u8),
    u16(u16),
    u32(u32),
    u64(u64),
    u128(u128),
    i8(i8),
    i16(i16),
    i32(i32),
    i64(i64),
    i128(i128),
    bool(bool),
    char(char),
    f32(f32),
    f64(f64),
    String(&'a str),
}

#[derive(Debug)]
pub enum ReflectMut<'a> {
    Struct(&'a mut dyn Struct),
    TupleStruct(&'a mut dyn TupleStruct),
    Tuple(&'a mut dyn Tuple),
    Enum(&'a mut dyn Enum),
    Array(&'a mut dyn Array),
    List(&'a mut dyn List),
    Map(&'a mut dyn Map),
    Scalar(ScalarMut<'a>),
    /// Not all `Reflect` implementations allow mutable access to the underlying value (such as
    /// [`core::num::NonZeroU8`]). This variant can be used for such types.
    Opaque(&'a mut dyn Reflect),
}

impl<'a> ReflectMut<'a> {
    pub fn as_tuple_mut(self) -> Option<&'a mut dyn Tuple> {
        match self {
            Self::Tuple(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_struct_mut(self) -> Option<&'a mut dyn Struct> {
        match self {
            Self::Struct(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_tuple_struct_mut(self) -> Option<&'a mut dyn TupleStruct> {
        match self {
            Self::TupleStruct(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_enum_mut(self) -> Option<&'a mut dyn Enum> {
        match self {
            Self::Enum(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_list_mut(self) -> Option<&'a mut dyn List> {
        match self {
            Self::List(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_array_mut(self) -> Option<&'a mut dyn Array> {
        match self {
            Self::Array(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_map_mut(self) -> Option<&'a mut dyn Map> {
        match self {
            Self::Map(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_scalar_mut(self) -> Option<ScalarMut<'a>> {
        match self {
            Self::Scalar(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_opaque_mut(self) -> Option<&'a mut dyn Reflect> {
        match self {
            Self::Opaque(inner) => Some(inner),
            _ => None,
        }
    }
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum ScalarMut<'a> {
    usize(&'a mut usize),
    u8(&'a mut u8),
    u16(&'a mut u16),
    u32(&'a mut u32),
    u64(&'a mut u64),
    u128(&'a mut u128),
    i8(&'a mut i8),
    i16(&'a mut i16),
    i32(&'a mut i32),
    i64(&'a mut i64),
    i128(&'a mut i128),
    bool(&'a mut bool),
    char(&'a mut char),
    f32(&'a mut f32),
    f64(&'a mut f64),
    String(&'a mut String),
}

/// Private. Used by macros
#[doc(hidden)]
pub mod __private {
    pub use alloc::collections::BTreeMap;
    pub use core::any::Any;
    pub use core::fmt;

    pub use self::enum_::*;
    pub use self::key_path::*;
    pub use self::struct_::*;
    pub use self::tuple::*;
    pub use self::tuple_struct::*;
    pub use self::value::*;
    pub use crate::iter::*;
    pub use crate::type_info::graph::*;
    pub use crate::*;

    pub trait IntoValue {
        fn into_value(self) -> Value;
    }

    impl<R> IntoValue for R
    where
        R: Reflect,
    {
        fn into_value(self) -> Value {
            self.to_value()
        }
    }

    impl IntoValue for &str {
        fn into_value(self) -> Value {
            self.to_owned().into_value()
        }
    }
}
