use alloc::borrow::ToOwned;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;
use core::iter::Peekable;

use crate::enum_::VariantKind;
use crate::type_info::TypeAtPath;
use crate::Reflect;
use crate::ReflectMut;
use crate::ReflectRef;

pub trait GetPath {
    fn at(&self, key_path: &KeyPath) -> Option<&dyn Reflect>;

    fn get_at<T>(&self, key_path: &KeyPath) -> Option<&T>
    where
        T: Reflect,
    {
        self.at(key_path)?.downcast_ref()
    }

    fn at_mut(&mut self, key_path: &KeyPath) -> Option<&mut dyn Reflect>;

    fn get_at_mut<T>(&mut self, key_path: &KeyPath) -> Option<&mut T>
    where
        T: Reflect,
    {
        self.at_mut(key_path)?.downcast_mut()
    }
}

pub trait GetTypePath<'a> {
    fn at_type(self, key_path: &KeyPath) -> Option<TypeAtPath<'a>>;
}

impl<R> GetPath for R
where
    R: Reflect + ?Sized,
{
    fn at(&self, key_path: &KeyPath) -> Option<&dyn Reflect> {
        fn go<'a, 'b, R>(
            value: &'a R,
            mut stack: Peekable<impl Iterator<Item = &'b Key>>,
        ) -> Option<&'a dyn Reflect>
        where
            R: Reflect + ?Sized,
        {
            let head = stack.next()?;

            let value_at_key = match head {
                // .foo
                Key::Field(private::KeyOrIndex::Key(key)) => match value.reflect_ref() {
                    ReflectRef::Struct(inner) => inner.field(key)?,
                    ReflectRef::Enum(inner) => match inner.variant_kind() {
                        VariantKind::Struct => inner.field(key)?,
                        VariantKind::Tuple | VariantKind::Unit => return None,
                    },
                    ReflectRef::TupleStruct(_)
                    | ReflectRef::Tuple(_)
                    | ReflectRef::Array(_)
                    | ReflectRef::List(_)
                    | ReflectRef::Map(_)
                    | ReflectRef::Scalar(_)
                    | ReflectRef::Opaque(_) => return None,
                },
                // .0
                Key::Field(private::KeyOrIndex::Index(index)) => match value.reflect_ref() {
                    ReflectRef::TupleStruct(inner) => inner.field_at(*index)?,
                    ReflectRef::Tuple(inner) => inner.field_at(*index)?,
                    ReflectRef::Enum(inner) => match inner.variant_kind() {
                        VariantKind::Tuple => inner.field_at(*index)?,
                        VariantKind::Struct | VariantKind::Unit => return None,
                    },
                    ReflectRef::Map(_)
                    | ReflectRef::Struct(_)
                    | ReflectRef::Array(_)
                    | ReflectRef::List(_)
                    | ReflectRef::Scalar(_)
                    | ReflectRef::Opaque(_) => return None,
                },
                // ["foo"]
                Key::FieldAt(private::KeyOrIndex::Key(key)) => match value.reflect_ref() {
                    ReflectRef::Map(inner) => inner.get(key)?,
                    ReflectRef::Struct(_)
                    | ReflectRef::TupleStruct(_)
                    | ReflectRef::Tuple(_)
                    | ReflectRef::Enum(_)
                    | ReflectRef::Array(_)
                    | ReflectRef::List(_)
                    | ReflectRef::Scalar(_)
                    | ReflectRef::Opaque(_) => return None,
                },
                // [0]
                Key::FieldAt(private::KeyOrIndex::Index(index)) => match value.reflect_ref() {
                    ReflectRef::Array(inner) => inner.get(*index)?,
                    ReflectRef::List(inner) => inner.get(*index)?,
                    ReflectRef::Map(inner) => inner.get(index)?,
                    ReflectRef::Struct(_)
                    | ReflectRef::TupleStruct(_)
                    | ReflectRef::Tuple(_)
                    | ReflectRef::Enum(_)
                    | ReflectRef::Scalar(_)
                    | ReflectRef::Opaque(_) => return None,
                },
                // ::Some
                Key::Variant(variant) => match value.reflect_ref() {
                    ReflectRef::Enum(enum_) => {
                        if enum_.variant_name() == variant {
                            enum_.as_reflect()
                        } else {
                            return None;
                        }
                    }
                    ReflectRef::Struct(_)
                    | ReflectRef::TupleStruct(_)
                    | ReflectRef::Tuple(_)
                    | ReflectRef::List(_)
                    | ReflectRef::Array(_)
                    | ReflectRef::Map(_)
                    | ReflectRef::Opaque(_)
                    | ReflectRef::Scalar(_) => return None,
                },
            };

            if stack.peek().is_none() {
                Some(value_at_key)
            } else {
                go(value_at_key, stack)
            }
        }

        if key_path.is_empty() {
            return Some(self.as_reflect());
        }

        go(self, key_path.path.iter().peekable())
    }

    fn at_mut(&mut self, key_path: &KeyPath) -> Option<&mut dyn Reflect> {
        fn go<'a, 'b, R>(
            value: &'a mut R,
            mut stack: Peekable<impl Iterator<Item = &'b Key>>,
        ) -> Option<&'a mut dyn Reflect>
        where
            R: Reflect + ?Sized,
        {
            let head = stack.next()?;

            let value_at_key = match head {
                // .foo
                Key::Field(private::KeyOrIndex::Key(key)) => match value.reflect_mut() {
                    ReflectMut::Struct(inner) => inner.field_mut(key)?,
                    ReflectMut::Enum(inner) => match inner.variant_kind() {
                        VariantKind::Struct => inner.field_mut(key)?,
                        VariantKind::Tuple | VariantKind::Unit => return None,
                    },
                    ReflectMut::TupleStruct(_)
                    | ReflectMut::Tuple(_)
                    | ReflectMut::Array(_)
                    | ReflectMut::List(_)
                    | ReflectMut::Map(_)
                    | ReflectMut::Scalar(_)
                    | ReflectMut::Opaque(_) => return None,
                },
                // .0
                Key::Field(private::KeyOrIndex::Index(index)) => match value.reflect_mut() {
                    ReflectMut::TupleStruct(inner) => inner.field_at_mut(*index)?,
                    ReflectMut::Tuple(inner) => inner.field_at_mut(*index)?,
                    ReflectMut::Enum(inner) => match inner.variant_kind() {
                        VariantKind::Tuple => inner.field_at_mut(*index)?,
                        VariantKind::Struct | VariantKind::Unit => return None,
                    },
                    ReflectMut::Map(_)
                    | ReflectMut::Struct(_)
                    | ReflectMut::Array(_)
                    | ReflectMut::List(_)
                    | ReflectMut::Scalar(_)
                    | ReflectMut::Opaque(_) => return None,
                },
                // ["foo"]
                Key::FieldAt(private::KeyOrIndex::Key(key)) => match value.reflect_mut() {
                    ReflectMut::Map(inner) => inner.get_mut(key)?,
                    ReflectMut::Struct(_)
                    | ReflectMut::TupleStruct(_)
                    | ReflectMut::Tuple(_)
                    | ReflectMut::Enum(_)
                    | ReflectMut::Array(_)
                    | ReflectMut::List(_)
                    | ReflectMut::Scalar(_)
                    | ReflectMut::Opaque(_) => return None,
                },
                // [0]
                Key::FieldAt(private::KeyOrIndex::Index(index)) => match value.reflect_mut() {
                    ReflectMut::Array(inner) => inner.get_mut(*index)?,
                    ReflectMut::List(inner) => inner.get_mut(*index)?,
                    ReflectMut::Map(inner) => inner.get_mut(index)?,
                    ReflectMut::Struct(_)
                    | ReflectMut::TupleStruct(_)
                    | ReflectMut::Tuple(_)
                    | ReflectMut::Enum(_)
                    | ReflectMut::Scalar(_)
                    | ReflectMut::Opaque(_) => return None,
                },
                // ::Some
                Key::Variant(variant) => match value.reflect_mut() {
                    ReflectMut::Enum(enum_) => {
                        if enum_.variant_name() == variant {
                            enum_.as_reflect_mut()
                        } else {
                            return None;
                        }
                    }
                    ReflectMut::Struct(_)
                    | ReflectMut::TupleStruct(_)
                    | ReflectMut::Tuple(_)
                    | ReflectMut::List(_)
                    | ReflectMut::Array(_)
                    | ReflectMut::Map(_)
                    | ReflectMut::Opaque(_)
                    | ReflectMut::Scalar(_) => return None,
                },
            };

            if stack.peek().is_none() {
                Some(value_at_key)
            } else {
                go(value_at_key, stack)
            }
        }

        if key_path.is_empty() {
            return Some(self.as_reflect_mut());
        }

        go(self, key_path.path.iter().peekable())
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "speedy", derive(speedy::Readable, speedy::Writable))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KeyPath {
    pub(crate) path: Vec<Key>,
}

impl KeyPath {
    pub fn field(mut self, field: impl IntoKeyOrIndex) -> Self {
        self.push_field(field);
        self
    }

    pub fn push_field(&mut self, field: impl IntoKeyOrIndex) {
        self.path.push(Key::Field(field.into_key_or_index()));
    }

    pub fn get(mut self, field: impl IntoKeyOrIndex) -> Self {
        self.push_get(field);
        self
    }

    pub fn push_get(&mut self, field: impl IntoKeyOrIndex) {
        self.path.push(Key::FieldAt(field.into_key_or_index()))
    }

    pub fn variant<S>(mut self, variant: S) -> Self
    where
        S: Into<String>,
    {
        self.push_variant(variant);
        self
    }

    pub fn push_variant<S>(&mut self, variant: S)
    where
        S: Into<String>,
    {
        self.path.push(Key::Variant(variant.into()));
    }

    pub fn len(&self) -> usize {
        self.path.len()
    }

    pub fn is_empty(&self) -> bool {
        self.path.is_empty()
    }

    pub fn pop(&mut self) {
        self.path.pop();
    }
}

mod private {
    use super::*;

    pub trait Sealed {}
    impl Sealed for &str {}
    impl Sealed for String {}
    impl Sealed for usize {}

    #[derive(Debug, Clone)]
    #[cfg_attr(feature = "speedy", derive(speedy::Readable, speedy::Writable))]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[allow(unreachable_pub)]
    pub enum Key {
        Field(KeyOrIndex),
        FieldAt(KeyOrIndex),
        Variant(String),
    }

    #[derive(Debug, Clone)]
    #[cfg_attr(feature = "speedy", derive(speedy::Readable, speedy::Writable))]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub enum KeyOrIndex {
        Key(String),
        Index(usize),
    }
}

pub(crate) use private::Key;
pub(crate) use private::KeyOrIndex;

pub trait IntoKeyOrIndex: private::Sealed {
    fn into_key_or_index(self) -> private::KeyOrIndex;
}

impl IntoKeyOrIndex for &str {
    fn into_key_or_index(self) -> private::KeyOrIndex {
        private::KeyOrIndex::Key(self.to_owned())
    }
}

impl IntoKeyOrIndex for String {
    fn into_key_or_index(self) -> private::KeyOrIndex {
        private::KeyOrIndex::Key(self)
    }
}

impl IntoKeyOrIndex for usize {
    fn into_key_or_index(self) -> private::KeyOrIndex {
        private::KeyOrIndex::Index(self)
    }
}

pub fn field(field: impl IntoKeyOrIndex) -> KeyPath {
    KeyPath::default().field(field)
}

pub fn get(field: impl IntoKeyOrIndex) -> KeyPath {
    KeyPath::default().get(field)
}

pub fn variant<S>(variant: S) -> KeyPath
where
    S: Into<String>,
{
    KeyPath::default().variant(variant)
}

#[macro_export]
macro_rules! key_path {
    // base case
    (
        @go:
        $path:expr,
        [],
    ) => {
        $path
    };

    // recursive case (field)
    (
        @go:
        $path:expr,
        [ . $field:ident $($tt:tt)*],
    ) => {
        $crate::key_path!(
            @go:
            $path.field(stringify!($field)),
            [$($tt)*],
        )
    };

    // recursive case (field)
    (
        @go:
        $path:expr,
        [ . $field:literal $($tt:tt)*],
    ) => {
        $crate::key_path!(
            @go:
            $path.field($field),
            [$($tt)*],
        )
    };

    // recursive case (field)
    (
        @go:
        $path:expr,
        [ [$field:expr] $($tt:tt)*],
    ) => {
        $crate::key_path!(
            @go:
            $path.get($field),
            [$($tt)*],
        )
    };

    // recursive case (variant)
    (
        @go:
        $path:expr,
        [ :: $variant:ident $($tt:tt)*],
    ) => {
        $crate::key_path!(
            @go:
            $path.variant(stringify!($variant)),
            [$($tt)*],
        )
    };

    // on invalid syntax
    (
        @go:
        $path:expr,
        [$($tt:tt)*],
    ) => {
        compile_error!(concat!("Unexpected tokens ", stringify!($($tt)*)))
    };

    // entry point
    ( $($tt:tt)* ) => {
        $crate::key_path!(
            @go:
            $crate::key_path::KeyPath::default(),
            [$($tt)*],
        )
    };
}

impl fmt::Display for KeyPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for key in &self.path {
            match key {
                Key::Field(private::KeyOrIndex::Key(key)) => write!(f, ".{key}")?,
                Key::Field(private::KeyOrIndex::Index(index)) => write!(f, ".{index}")?,
                Key::FieldAt(private::KeyOrIndex::Key(key)) => write!(f, "[{key:?}]")?,
                Key::FieldAt(private::KeyOrIndex::Index(index)) => write!(f, "[{index}]")?,
                Key::Variant(variant) => write!(f, "::{variant}")?,
            }
        }
        Ok(())
    }
}
