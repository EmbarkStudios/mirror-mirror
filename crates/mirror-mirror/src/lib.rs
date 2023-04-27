//! General purpose reflection library for Rust.
//!
//! # Examples
//!
//! ## Access a field by its string name and mutate it
//!
//! ```
//! use mirror_mirror::{Reflect, Struct};
//!
//! #[derive(Reflect, Clone, Debug)]
//! struct Foo {
//!     x: i32,
//! }
//!
//! let mut foo = Foo { x: 42 };
//!
//! # (|| {
//! // Get a `Struct` trait object for `Foo`.
//! //
//! // The `Struct` trait has methods available for all structs such as accessing
//! // fields by name and iterating over the fields.
//! let struct_obj: &mut dyn Struct = foo.as_struct_mut()?;
//!
//! // Mutably borrow the `x` field. We can access fields using string names.
//! let x: &mut dyn Reflect = struct_obj.field_mut("x")?;
//!
//! // Downcast `x` into a mutable `i32`
//! let x: &mut i32 = x.downcast_mut::<i32>()?;
//!
//! // Change the value of `x`
//! *x += 1;
//!
//! // The value of `x` in `foo` has now changed.
//! assert_eq!(foo.x, 43);
//! # Some(())
//! # })().unwrap();
//! ```
//!
//! ## Iterate over all fields
//!
//! ```
//! use mirror_mirror::{Reflect, Struct, ReflectMut, ScalarMut, enum_::VariantFieldMut};
//!
//! // A function that iterates over the fields in an enum and mutates them.
//! fn change_enum_fields(value: &mut dyn Reflect) -> Option<()> {
//!     let enum_ = value.as_enum_mut()?;
//!
//!     for field in enum_.fields_mut() {
//!         match field {
//!             VariantFieldMut::Struct(_, value) | VariantFieldMut::Tuple(value) => {
//!                 match value.reflect_mut() {
//!                     ReflectMut::Scalar(ScalarMut::i32(n)) => {
//!                         *n *= 2;
//!                     }
//!                     ReflectMut::Scalar(ScalarMut::String(s)) => {
//!                         *s = format!("{s}bar");
//!                     }
//!                     // Ignore other types
//!                     _ =>  {}
//!                 }
//!             }
//!         }
//!     }
//!
//!     Some(())
//! }
//!
//! #[derive(Reflect, Clone, Debug)]
//! enum Bar {
//!     X { x: i32 },
//!     Y(String),
//! }
//!
//! # (|| {
//! let mut bar = Bar::X { x: 42 };
//! change_enum_fields(bar.as_reflect_mut())?;
//!
//! assert!(matches!(bar, Bar::X { x: 84 }));
//!
//! let mut bar = Bar::Y("foo".to_owned());
//! change_enum_fields(bar.as_reflect_mut())?;
//!
//! assert!(matches!(bar, Bar::Y(s) if s == "foobar"));
//! # Some(())
//! # })().unwrap();
//! ```
//!
//! ## Query value and type information using key paths
//!
//! ```
//! use mirror_mirror::{
//!     Reflect,
//!     key_path,
//!     key_path::{GetPath, GetTypePath, field},
//!     type_info::{DescribeType, ScalarType},
//! };
//!
//! // Some complex nested data type.
//! #[derive(Reflect, Clone, Debug)]
//! struct User {
//!     employer: Option<Company>,
//! }
//!
//! #[derive(Reflect, Clone, Debug)]
//! struct Company {
//!     countries: Vec<Country>,
//! }
//!
//! #[derive(Reflect, Clone, Debug)]
//! struct Country {
//!     name: String
//! }
//!
//! let user = User {
//!     employer: Some(Company {
//!         countries: vec![Country {
//!             name: "Denmark".to_owned(),
//!         }],
//!     }),
//! };
//!
//! // Build a key path that represents accessing `.employer::Some.0.countries[0].name`.
//! //
//! // `::Some` means to access the `Some` variant of `Option<Company>`.
//! let path = field("employer").variant("Some").field(0).field("countries").get(0).field("name");
//!
//! // Get the value at the key path.
//! assert_eq!(user.get_at::<String>(&path).unwrap(), "Denmark");
//!
//! // Key paths can also be constructed using the `key_path!` macro.
//! // This invocation expands the same code we have above.
//! let path = key_path!(.employer::Some.0.countries[0].name);
//!
//! // Use the same key path to query type information. You don't need a value
//! // of the type to access its type information.
//! let user_type = <User as DescribeType>::type_descriptor();
//! assert!(matches!(
//!     user_type.type_at(&path).unwrap().as_scalar().unwrap(),
//!     ScalarType::String,
//! ));
//! ```
//!
//! ## Using opaque `Value` types
//!
//! ```
//! use mirror_mirror::{Reflect, Value, FromReflect};
//!
//! #[derive(Reflect, Clone, Debug)]
//! struct Foo(Vec<i32>);
//!
//! # (|| {
//! let foo = Foo(vec![1, 2, 3]);
//!
//! // Convert `foo` into general "value" type.
//! let mut value: Value = foo.to_value();
//!
//! // `Value` also implements `Reflect` so it can be mutated in the
//! // same way we've seen before. So these mutations can be made
//! // by another crate that doesn't know about the `Foo` type.
//! //
//! // `Value` is also serializable with `speedy`, for binary serialization,
//! // or `serde`, for everything else.
//! value
//!     .as_tuple_struct_mut()?
//!     .field_at_mut(0)?
//!     .as_list_mut()?
//!     .push(&4);
//!
//! // Convert the `value` back into a `Foo`.
//! let new_foo = Foo::from_reflect(&value)?;
//!
//! // Our changes were applied.
//! assert_eq!(new_foo.0, vec![1, 2, 3, 4]);
//! # Some(())
//! # })().unwrap();
//! ```
//!
//! # Inspiration
//!
//! The design of this library is heavily inspired by [`bevy_reflect`] but with a few key
//! differences:
//!
//! - [`speedy`] integration which is useful for marshalling data perhaps to send it across FFI.
//! - A [`Value`] type that can be serialized and deserialized without using trait objects.
//! - More [type information][type_info] captured.
//! - Add meta data to types which becomes part of the type information.
//! - [Key paths][mod@key_path] for querying value and type information.
//! - No dependencies on [`bevy`] specific crates.
//! - `#![no_std]` support.
//!
//! # Feature flags
//!
//! mirror-mirror uses a set of [feature flags] to optionally reduce the number of dependencies.
//!
//! The following optional features are available:
//!
//! Name | Description | Default?
//! ---|---|---
//! `std` | Enables using the standard library (`core` and `alloc` are always required) | Yes
//! `speedy` | Enables [`speedy`] support for most types | Yes
//! `serde` | Enables [`serde`] support for most types | Yes
//! `glam` | Enables impls for [`glam`] | No
//! `macaw` | Enables impls for [`macaw`] | No
//!
//! [`speedy`]: https://crates.io/crates/speedy
//! [`serde`]: https://crates.io/crates/serde
//! [`bevy_reflect`]: https://crates.io/crates/bevy_reflect
//! [`bevy`]: https://crates.io/crates/bevy
//! [`glam`]: https://crates.io/crates/glam
//! [`macaw`]: https://crates.io/crates/macaw

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
    missing_debug_implementations,
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
use core::fmt;

use crate::enum_::VariantField;
use crate::enum_::VariantKind;

macro_rules! trivial_reflect_methods {
    () => {
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
    };
}

/// Reflected array types.
pub mod array;

/// Reflected enum types.
pub mod enum_;

/// Helper traits for accessing fields on reflected values.
pub mod get_field;

/// Iterator types.
pub mod iter;

/// Key paths for querying value and type information.
pub mod key_path;

/// Reflected list types.
pub mod list;

/// Reflected map types.
pub mod map;

/// Reflected struct types.
pub mod struct_;

/// Reflected tuple types.
pub mod tuple;

/// Reflected tuple struct types.
pub mod tuple_struct;

/// Type information.
pub mod type_info;

/// Type erased value types.
pub mod value;

pub mod try_visit;

mod foreign_impls;

#[cfg(feature = "simple_type_name")]
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
pub use self::type_info::DescribeType;
#[doc(inline)]
pub use self::type_info::TypeDescriptor;
#[doc(inline)]
pub use self::value::Value;

pub(crate) static STATIC_RANDOM_STATE: ahash::RandomState = ahash::RandomState::with_seeds(
    0x86c11a44c63f4f2f,
    0xaf04d821054d02b3,
    0x98f0a276c462acc1,
    0xe2d6368e09c9c079,
);

/// A reflected type.
pub trait Reflect: Any + Send + 'static {
    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn as_reflect(&self) -> &dyn Reflect;

    fn as_reflect_mut(&mut self) -> &mut dyn Reflect;

    fn reflect_owned(self: Box<Self>) -> ReflectOwned;

    fn reflect_ref(&self) -> ReflectRef<'_>;

    fn reflect_mut(&mut self) -> ReflectMut<'_>;

    fn patch(&mut self, value: &dyn Reflect);

    fn to_value(&self) -> Value;

    fn clone_reflect(&self) -> Box<dyn Reflect>;

    fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;

    /// Get the type name of the value.
    ///
    /// ```
    /// use mirror_mirror::Reflect;
    ///
    /// fn dyn_reflect_type_name(value: &dyn Reflect) -> &str {
    ///     value.type_name()
    /// }
    ///
    /// assert_eq!(dyn_reflect_type_name(&1_i32), "i32");
    /// ```
    ///
    /// Note that converting something into a [`Value`] will change what this method returns:
    ///
    /// ```
    /// use mirror_mirror::Reflect;
    ///
    /// fn dyn_reflect_type_name(value: &dyn Reflect) -> &str {
    ///     value.type_name()
    /// }
    ///
    /// assert_eq!(
    ///     dyn_reflect_type_name(&1_i32.to_value()),
    ///     // the type name is no longer "i32"
    ///     "mirror_mirror::value::Value",
    /// );
    /// ```
    ///
    /// If you want to keep the name of the original type use [`DescribeType::type_descriptor`].
    fn type_name(&self) -> &str {
        core::any::type_name::<Self>()
    }

    fn into_tuple(self: Box<Self>) -> Option<Box<dyn Tuple>> {
        self.reflect_owned().into_tuple()
    }

    fn as_tuple(&self) -> Option<&dyn Tuple> {
        self.reflect_ref().as_tuple()
    }

    fn as_tuple_mut(&mut self) -> Option<&mut dyn Tuple> {
        self.reflect_mut().as_tuple_mut()
    }

    fn into_struct(self: Box<Self>) -> Option<Box<dyn Struct>> {
        self.reflect_owned().into_struct()
    }

    fn as_struct(&self) -> Option<&dyn Struct> {
        self.reflect_ref().as_struct()
    }

    fn as_struct_mut(&mut self) -> Option<&mut dyn Struct> {
        self.reflect_mut().as_struct_mut()
    }

    fn into_tuple_struct(self: Box<Self>) -> Option<Box<dyn TupleStruct>> {
        self.reflect_owned().into_tuple_struct()
    }

    fn as_tuple_struct(&self) -> Option<&dyn TupleStruct> {
        self.reflect_ref().as_tuple_struct()
    }

    fn as_tuple_struct_mut(&mut self) -> Option<&mut dyn TupleStruct> {
        self.reflect_mut().as_tuple_struct_mut()
    }

    fn into_enum(self: Box<Self>) -> Option<Box<dyn Enum>> {
        self.reflect_owned().into_enum()
    }

    fn as_enum(&self) -> Option<&dyn Enum> {
        self.reflect_ref().as_enum()
    }

    fn as_enum_mut(&mut self) -> Option<&mut dyn Enum> {
        self.reflect_mut().as_enum_mut()
    }

    fn into_list(self: Box<Self>) -> Option<Box<dyn List>> {
        self.reflect_owned().into_list()
    }

    fn as_list(&self) -> Option<&dyn List> {
        self.reflect_ref().as_list()
    }

    fn as_list_mut(&mut self) -> Option<&mut dyn List> {
        self.reflect_mut().as_list_mut()
    }

    fn into_array(self: Box<Self>) -> Option<Box<dyn Array>> {
        self.reflect_owned().into_array()
    }

    fn as_array(&self) -> Option<&dyn Array> {
        self.reflect_ref().as_array()
    }

    fn as_array_mut(&mut self) -> Option<&mut dyn Array> {
        self.reflect_mut().as_array_mut()
    }

    fn into_map(self: Box<Self>) -> Option<Box<dyn Map>> {
        self.reflect_owned().into_map()
    }

    fn as_map(&self) -> Option<&dyn Map> {
        self.reflect_ref().as_map()
    }

    fn as_map_mut(&mut self) -> Option<&mut dyn Map> {
        self.reflect_mut().as_map_mut()
    }

    fn into_scalar(self: Box<Self>) -> Option<ScalarOwned> {
        self.reflect_owned().into_scalar()
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

impl ToOwned for dyn Reflect {
    type Owned = Box<dyn Reflect>;

    fn to_owned(&self) -> Self::Owned {
        self.clone_reflect()
    }
}

impl fmt::Debug for dyn Reflect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.debug(f)
    }
}

macro_rules! impl_for_core_types {
    ($($ty:ident)*) => {
        $(
            impl Reflect for $ty {
                trivial_reflect_methods!();

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

                fn reflect_owned(self: Box<Self>) -> ReflectOwned {
                    ReflectOwned::Scalar(ScalarOwned::from(*self))
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

            impl From<$ty> for ScalarOwned {
                fn from(value: $ty) -> Self {
                    ScalarOwned::$ty(value)
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
    trivial_reflect_methods!();

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

    fn reflect_owned(self: Box<Self>) -> ReflectOwned {
        ReflectOwned::Scalar(ScalarOwned::from(*self))
    }

    fn reflect_ref(&self) -> ReflectRef<'_> {
        ReflectRef::Scalar(ScalarRef::from(self))
    }

    fn reflect_mut(&mut self) -> ReflectMut<'_> {
        ReflectMut::Scalar(ScalarMut::String(self))
    }

    fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "{self:#?}")
        } else {
            write!(f, "{self:?}")
        }
    }
}

impl FromReflect for String {
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
        Some(reflect.downcast_ref::<String>()?.clone())
    }
}

impl From<String> for ScalarOwned {
    fn from(value: String) -> Self {
        ScalarOwned::String(value)
    }
}

impl<'a> From<&'a String> for ScalarRef<'a> {
    fn from(value: &'a String) -> Self {
        ScalarRef::String(value)
    }
}

impl<'a> From<&'a mut String> for ScalarMut<'a> {
    fn from(value: &'a mut String) -> Self {
        ScalarMut::String(value)
    }
}

/// A trait for types which can be constructed from a reflected type.
///
/// Will be implemented by `#[derive(Reflect)]`.
pub trait FromReflect: Reflect + Sized {
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self>;
}

/// An owned reflected value.
///
/// Constructed with [`Reflect::reflect_owned`].
#[derive(Debug)]
pub enum ReflectOwned {
    Struct(Box<dyn Struct>),
    TupleStruct(Box<dyn TupleStruct>),
    Tuple(Box<dyn Tuple>),
    Enum(Box<dyn Enum>),
    Array(Box<dyn Array>),
    List(Box<dyn List>),
    Map(Box<dyn Map>),
    Scalar(ScalarOwned),
    /// Not all `Reflect` implementations allow access to the underlying value. This variant can be
    /// used for such types.
    Opaque(Box<dyn Reflect>),
}

impl ReflectOwned {
    pub fn as_reflect_mut(&mut self) -> &mut dyn Reflect {
        match self {
            ReflectOwned::Struct(inner) => inner.as_reflect_mut(),
            ReflectOwned::TupleStruct(inner) => inner.as_reflect_mut(),
            ReflectOwned::Tuple(inner) => inner.as_reflect_mut(),
            ReflectOwned::Enum(inner) => inner.as_reflect_mut(),
            ReflectOwned::Array(inner) => inner.as_reflect_mut(),
            ReflectOwned::List(inner) => inner.as_reflect_mut(),
            ReflectOwned::Map(inner) => inner.as_reflect_mut(),
            ReflectOwned::Scalar(inner) => inner.as_reflect_mut(),
            ReflectOwned::Opaque(inner) => inner.as_reflect_mut(),
        }
    }

    pub fn as_reflect(&self) -> &dyn Reflect {
        match self {
            ReflectOwned::Struct(inner) => inner.as_reflect(),
            ReflectOwned::TupleStruct(inner) => inner.as_reflect(),
            ReflectOwned::Tuple(inner) => inner.as_reflect(),
            ReflectOwned::Enum(inner) => inner.as_reflect(),
            ReflectOwned::Array(inner) => inner.as_reflect(),
            ReflectOwned::List(inner) => inner.as_reflect(),
            ReflectOwned::Map(inner) => inner.as_reflect(),
            ReflectOwned::Scalar(inner) => inner.as_reflect(),
            ReflectOwned::Opaque(inner) => inner.as_reflect(),
        }
    }

    pub fn into_tuple(self) -> Option<Box<dyn Tuple>> {
        match self {
            Self::Tuple(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn into_struct(self) -> Option<Box<dyn Struct>> {
        match self {
            Self::Struct(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn into_tuple_struct(self) -> Option<Box<dyn TupleStruct>> {
        match self {
            Self::TupleStruct(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn into_enum(self) -> Option<Box<dyn Enum>> {
        match self {
            Self::Enum(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn into_list(self) -> Option<Box<dyn List>> {
        match self {
            Self::List(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn into_array(self) -> Option<Box<dyn Array>> {
        match self {
            Self::Array(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn into_map(self) -> Option<Box<dyn Map>> {
        match self {
            Self::Map(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn into_scalar(self) -> Option<ScalarOwned> {
        match self {
            Self::Scalar(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn into_opaque(self) -> Option<Box<dyn Reflect>> {
        match self {
            Self::Opaque(inner) => Some(inner),
            _ => None,
        }
    }
}

impl Clone for ReflectOwned {
    fn clone(&self) -> Self {
        match self {
            Self::Struct(inner) => inner.clone_reflect().reflect_owned(),
            Self::TupleStruct(inner) => inner.clone_reflect().reflect_owned(),
            Self::Tuple(inner) => inner.clone_reflect().reflect_owned(),
            Self::Enum(inner) => inner.clone_reflect().reflect_owned(),
            Self::Array(inner) => inner.clone_reflect().reflect_owned(),
            Self::List(inner) => inner.clone_reflect().reflect_owned(),
            Self::Map(inner) => inner.clone_reflect().reflect_owned(),
            Self::Opaque(inner) => inner.clone_reflect().reflect_owned(),
            Self::Scalar(inner) => Self::Scalar(inner.clone()),
        }
    }
}

/// An owned reflected scalar type.
#[derive(Debug, Clone)]
#[allow(non_camel_case_types)]
#[cfg_attr(feature = "speedy", derive(speedy::Readable, speedy::Writable))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ScalarOwned {
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
    String(String),
}

impl ScalarOwned {
    pub fn as_reflect_mut(&mut self) -> &mut dyn Reflect {
        match self {
            ScalarOwned::usize(inner) => inner,
            ScalarOwned::u8(inner) => inner,
            ScalarOwned::u16(inner) => inner,
            ScalarOwned::u32(inner) => inner,
            ScalarOwned::u64(inner) => inner,
            ScalarOwned::u128(inner) => inner,
            ScalarOwned::i8(inner) => inner,
            ScalarOwned::i16(inner) => inner,
            ScalarOwned::i32(inner) => inner,
            ScalarOwned::i64(inner) => inner,
            ScalarOwned::i128(inner) => inner,
            ScalarOwned::bool(inner) => inner,
            ScalarOwned::char(inner) => inner,
            ScalarOwned::f32(inner) => inner,
            ScalarOwned::f64(inner) => inner,
            ScalarOwned::String(inner) => inner,
        }
    }

    pub fn as_reflect(&self) -> &dyn Reflect {
        match self {
            ScalarOwned::usize(inner) => inner,
            ScalarOwned::u8(inner) => inner,
            ScalarOwned::u16(inner) => inner,
            ScalarOwned::u32(inner) => inner,
            ScalarOwned::u64(inner) => inner,
            ScalarOwned::u128(inner) => inner,
            ScalarOwned::i8(inner) => inner,
            ScalarOwned::i16(inner) => inner,
            ScalarOwned::i32(inner) => inner,
            ScalarOwned::i64(inner) => inner,
            ScalarOwned::i128(inner) => inner,
            ScalarOwned::bool(inner) => inner,
            ScalarOwned::char(inner) => inner,
            ScalarOwned::f32(inner) => inner,
            ScalarOwned::f64(inner) => inner,
            ScalarOwned::String(inner) => inner,
        }
    }
}

/// An immutable reflected value.
///
/// Constructed with [`Reflect::reflect_ref`].
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
    pub fn as_reflect(&self) -> &dyn Reflect {
        match self {
            ReflectRef::Struct(inner) => inner.as_reflect(),
            ReflectRef::TupleStruct(inner) => inner.as_reflect(),
            ReflectRef::Tuple(inner) => inner.as_reflect(),
            ReflectRef::Enum(inner) => inner.as_reflect(),
            ReflectRef::Array(inner) => inner.as_reflect(),
            ReflectRef::List(inner) => inner.as_reflect(),
            ReflectRef::Map(inner) => inner.as_reflect(),
            ReflectRef::Scalar(inner) => inner.as_reflect(),
            ReflectRef::Opaque(inner) => inner.as_reflect(),
        }
    }

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

/// An immutable reflected scalar value.
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
    String(&'a String),
}

impl<'a> ScalarRef<'a> {
    pub fn as_reflect(&self) -> &dyn Reflect {
        match self {
            ScalarRef::usize(inner) => inner,
            ScalarRef::u8(inner) => inner,
            ScalarRef::u16(inner) => inner,
            ScalarRef::u32(inner) => inner,
            ScalarRef::u64(inner) => inner,
            ScalarRef::u128(inner) => inner,
            ScalarRef::i8(inner) => inner,
            ScalarRef::i16(inner) => inner,
            ScalarRef::i32(inner) => inner,
            ScalarRef::i64(inner) => inner,
            ScalarRef::i128(inner) => inner,
            ScalarRef::bool(inner) => inner,
            ScalarRef::char(inner) => inner,
            ScalarRef::f32(inner) => inner,
            ScalarRef::f64(inner) => inner,
            ScalarRef::String(inner) => *inner,
        }
    }
}

/// A mutable reflected value.
///
/// Constructed with [`Reflect::reflect_mut`].
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
    pub fn as_reflect_mut(&mut self) -> &mut dyn Reflect {
        match self {
            ReflectMut::Struct(inner) => inner.as_reflect_mut(),
            ReflectMut::TupleStruct(inner) => inner.as_reflect_mut(),
            ReflectMut::Tuple(inner) => inner.as_reflect_mut(),
            ReflectMut::Enum(inner) => inner.as_reflect_mut(),
            ReflectMut::Array(inner) => inner.as_reflect_mut(),
            ReflectMut::List(inner) => inner.as_reflect_mut(),
            ReflectMut::Map(inner) => inner.as_reflect_mut(),
            ReflectMut::Scalar(inner) => inner.as_reflect_mut(),
            ReflectMut::Opaque(inner) => inner.as_reflect_mut(),
        }
    }

    pub fn as_reflect(&self) -> &dyn Reflect {
        match self {
            ReflectMut::Struct(inner) => inner.as_reflect(),
            ReflectMut::TupleStruct(inner) => inner.as_reflect(),
            ReflectMut::Tuple(inner) => inner.as_reflect(),
            ReflectMut::Enum(inner) => inner.as_reflect(),
            ReflectMut::Array(inner) => inner.as_reflect(),
            ReflectMut::List(inner) => inner.as_reflect(),
            ReflectMut::Map(inner) => inner.as_reflect(),
            ReflectMut::Scalar(inner) => inner.as_reflect(),
            ReflectMut::Opaque(inner) => inner.as_reflect(),
        }
    }

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

/// An mutable reflected scalar value.
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

impl<'a> ScalarMut<'a> {
    pub fn as_reflect_mut(&mut self) -> &mut dyn Reflect {
        match self {
            ScalarMut::usize(inner) => *inner,
            ScalarMut::u8(inner) => *inner,
            ScalarMut::u16(inner) => *inner,
            ScalarMut::u32(inner) => *inner,
            ScalarMut::u64(inner) => *inner,
            ScalarMut::u128(inner) => *inner,
            ScalarMut::i8(inner) => *inner,
            ScalarMut::i16(inner) => *inner,
            ScalarMut::i32(inner) => *inner,
            ScalarMut::i64(inner) => *inner,
            ScalarMut::i128(inner) => *inner,
            ScalarMut::bool(inner) => *inner,
            ScalarMut::char(inner) => *inner,
            ScalarMut::f32(inner) => *inner,
            ScalarMut::f64(inner) => *inner,
            ScalarMut::String(inner) => *inner,
        }
    }

    pub fn as_reflect(&self) -> &dyn Reflect {
        match self {
            ScalarMut::usize(inner) => *inner,
            ScalarMut::u8(inner) => *inner,
            ScalarMut::u16(inner) => *inner,
            ScalarMut::u32(inner) => *inner,
            ScalarMut::u64(inner) => *inner,
            ScalarMut::u128(inner) => *inner,
            ScalarMut::i8(inner) => *inner,
            ScalarMut::i16(inner) => *inner,
            ScalarMut::i32(inner) => *inner,
            ScalarMut::i64(inner) => *inner,
            ScalarMut::i128(inner) => *inner,
            ScalarMut::bool(inner) => *inner,
            ScalarMut::char(inner) => *inner,
            ScalarMut::f32(inner) => *inner,
            ScalarMut::f64(inner) => *inner,
            ScalarMut::String(inner) => *inner,
        }
    }
}

/// Debug formatter for any reflection value.
pub fn reflect_debug(value: &dyn Reflect, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    fn scalar_debug(
        scalar: &dyn core::fmt::Debug,
        f: &mut core::fmt::Formatter<'_>,
    ) -> fmt::Result {
        if f.alternate() {
            write!(f, "{scalar:#?}")
        } else {
            write!(f, "{scalar:?}")
        }
    }

    match value.reflect_ref() {
        ReflectRef::Struct(inner) => {
            let mut f = f.debug_struct(inner.type_name());
            for (name, value) in inner.fields() {
                f.field(name, &value as &dyn ::core::fmt::Debug);
            }
            f.finish()
        }
        ReflectRef::TupleStruct(inner) => {
            let mut f = f.debug_tuple(inner.type_name());
            for field in inner.fields() {
                f.field(&field as &dyn ::core::fmt::Debug);
            }
            f.finish()
        }
        ReflectRef::Tuple(inner) => {
            let mut f = f.debug_tuple("");
            for field in inner.fields() {
                f.field(&field as &dyn ::core::fmt::Debug);
            }
            f.finish()
        }
        ReflectRef::Enum(inner) => match inner.variant_kind() {
            VariantKind::Struct => {
                let mut f = f.debug_struct(inner.variant_name());
                for field in inner.fields() {
                    match field {
                        VariantField::Struct(name, value) => {
                            f.field(name, &value as &dyn ::core::fmt::Debug);
                        }
                        VariantField::Tuple { .. } => {
                            unreachable!("unit variant yielded struct field")
                        }
                    }
                }
                f.finish()
            }
            VariantKind::Tuple => {
                let mut f = f.debug_tuple(inner.variant_name());
                for field in inner.fields() {
                    match field {
                        VariantField::Struct { .. } => {
                            unreachable!("unit variant yielded struct field")
                        }
                        VariantField::Tuple(value) => {
                            f.field(&value as &dyn ::core::fmt::Debug);
                        }
                    }
                }
                f.finish()
            }
            VariantKind::Unit => write!(f, "{}", inner.variant_name()),
        },
        ReflectRef::Array(inner) => f.debug_list().entries(inner.iter()).finish(),
        ReflectRef::List(inner) => f.debug_list().entries(inner.iter()).finish(),
        ReflectRef::Map(inner) => f.debug_map().entries(inner.iter()).finish(),
        ReflectRef::Scalar(inner) => match inner {
            ScalarRef::usize(inner) => scalar_debug(&inner, f),
            ScalarRef::u8(inner) => scalar_debug(&inner, f),
            ScalarRef::u16(inner) => scalar_debug(&inner, f),
            ScalarRef::u32(inner) => scalar_debug(&inner, f),
            ScalarRef::u64(inner) => scalar_debug(&inner, f),
            ScalarRef::u128(inner) => scalar_debug(&inner, f),
            ScalarRef::i8(inner) => scalar_debug(&inner, f),
            ScalarRef::i16(inner) => scalar_debug(&inner, f),
            ScalarRef::i32(inner) => scalar_debug(&inner, f),
            ScalarRef::i64(inner) => scalar_debug(&inner, f),
            ScalarRef::i128(inner) => scalar_debug(&inner, f),
            ScalarRef::bool(inner) => scalar_debug(&inner, f),
            ScalarRef::char(inner) => scalar_debug(&inner, f),
            ScalarRef::f32(inner) => scalar_debug(&inner, f),
            ScalarRef::f64(inner) => scalar_debug(&inner, f),
            ScalarRef::String(inner) => scalar_debug(&inner, f),
        },
        ReflectRef::Opaque(_) => {
            write!(f, "{}", value.type_name())
        }
    }
}

/// Private. Used by macros
#[doc(hidden)]
pub mod __private {
    pub use alloc::borrow::Cow;
    pub use alloc::collections::BTreeMap;
    pub use core::any::Any;
    pub use core::any::TypeId;
    pub use core::fmt;

    pub use once_cell::race::OnceBox;

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
