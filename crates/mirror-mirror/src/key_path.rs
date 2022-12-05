use std::fmt;

use serde::Deserialize;
use serde::Serialize;
use speedy::Readable;
use speedy::Writable;

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

impl<R> GetPath for R
where
    R: Reflect + ?Sized,
{
    fn at(&self, key_path: &KeyPath) -> Option<&dyn Reflect> {
        fn go<'a, R>(value: &'a R, mut stack: Vec<&Key>) -> Option<&'a dyn Reflect>
        where
            R: Reflect + ?Sized,
        {
            let head = stack.pop()?;

            let value_at_key = match head {
                Key::Field(key) => match value.reflect_ref() {
                    ReflectRef::Struct(inner) => inner.field(key)?,
                    ReflectRef::Map(inner) => inner.get(&key.to_owned())?,
                    ReflectRef::Enum(inner) => inner.field(key)?,
                    ReflectRef::TupleStruct(_)
                    | ReflectRef::Tuple(_)
                    | ReflectRef::List(_)
                    | ReflectRef::Array(_)
                    | ReflectRef::Opaque(_)
                    | ReflectRef::Scalar(_) => return None,
                },
                Key::Element(index) => match value.reflect_ref() {
                    ReflectRef::TupleStruct(inner) => inner.field(*index)?,
                    ReflectRef::Tuple(inner) => inner.field(*index)?,
                    ReflectRef::Enum(inner) => inner.field_at(*index)?,
                    ReflectRef::List(inner) => inner.get(*index)?,
                    ReflectRef::Array(inner) => inner.get(*index)?,
                    ReflectRef::Map(inner) => inner.get(index)?,
                    ReflectRef::Struct(_) | ReflectRef::Scalar(_) | ReflectRef::Opaque(_) => {
                        return None
                    }
                },
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

            if stack.is_empty() {
                Some(value_at_key)
            } else {
                go(value_at_key, stack)
            }
        }

        if key_path.is_empty() {
            return Some(self.as_reflect());
        }

        let mut path = key_path.path.iter().collect::<Vec<_>>();
        path.reverse();
        go(self, path)
    }

    fn at_mut(&mut self, key_path: &KeyPath) -> Option<&mut dyn Reflect> {
        fn go<'a, R>(value: &'a mut R, mut stack: Vec<&Key>) -> Option<&'a mut dyn Reflect>
        where
            R: Reflect + ?Sized,
        {
            let head = stack.pop()?;

            let value_at_key = match head {
                Key::Field(key) => match value.reflect_mut() {
                    ReflectMut::Struct(inner) => inner.field_mut(key)?,
                    ReflectMut::Map(inner) => inner.get_mut(key)?,
                    ReflectMut::Enum(inner) => inner.field_mut(key)?,
                    ReflectMut::TupleStruct(_)
                    | ReflectMut::Tuple(_)
                    | ReflectMut::List(_)
                    | ReflectMut::Array(_)
                    | ReflectMut::Opaque(_)
                    | ReflectMut::Scalar(_) => return None,
                },
                Key::Element(index) => match value.reflect_mut() {
                    ReflectMut::TupleStruct(inner) => inner.field_mut(*index)?,
                    ReflectMut::Tuple(inner) => inner.field_mut(*index)?,
                    ReflectMut::Enum(inner) => inner.field_at_mut(*index)?,
                    ReflectMut::List(inner) => inner.get_mut(*index)?,
                    ReflectMut::Array(inner) => inner.get_mut(*index)?,
                    ReflectMut::Map(inner) => inner.get_mut(index)?,
                    ReflectMut::Struct(_) | ReflectMut::Scalar(_) | ReflectMut::Opaque(_) => {
                        return None
                    }
                },
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

            if stack.is_empty() {
                Some(value_at_key)
            } else {
                go(value_at_key, stack)
            }
        }

        if key_path.is_empty() {
            return Some(self.as_reflect_mut());
        }

        let mut path = key_path.path.iter().collect::<Vec<_>>();
        path.reverse();
        go(self, path)
    }
}

#[derive(Readable, Writable, Serialize, Deserialize, Debug, Clone, Default)]
pub struct KeyPath {
    pub(crate) path: Vec<Key>,
}

impl KeyPath {
    pub fn field(mut self, field: impl IntoKey) -> Self {
        self.push_field(field);
        self
    }

    pub fn push_field(&mut self, field: impl IntoKey) {
        self.path.push(field.into_key());
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

    #[derive(Readable, Writable, Serialize, Deserialize, Debug, Clone)]
    pub enum Key {
        Field(String),
        Element(usize),
        Variant(String),
    }
}

pub(crate) use private::Key;

pub trait IntoKey: private::Sealed {
    fn into_key(self) -> Key;
}

impl IntoKey for &str {
    fn into_key(self) -> Key {
        Key::Field(self.to_owned())
    }
}

impl IntoKey for String {
    fn into_key(self) -> Key {
        Key::Field(self)
    }
}

impl IntoKey for usize {
    fn into_key(self) -> Key {
        Key::Element(self)
    }
}

pub fn field(field: impl IntoKey) -> KeyPath {
    KeyPath::default().field(field)
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
    ) => {{
        $path
    }};

    // recursive case (field)
    (
        @go:
        $path:expr,
        [ . $field:ident $($tt:tt)*],
    ) => {{
        $crate::key_path!(
            @go:
            $path.field(stringify!($field)),
            [$($tt)*],
        )
    }};

    // recursive case (field)
    (
        @go:
        $path:expr,
        [ [$field:expr] $($tt:tt)*],
    ) => {{
        $crate::key_path!(
            @go:
            $path.field($field),
            [$($tt)*],
        )
    }};

    // recursive case (variant)
    (
        @go:
        $path:expr,
        [ {$variant:ident} $($tt:tt)*],
    ) => {{
        $crate::key_path!(
            @go:
            $path.variant(stringify!($variant)),
            [$($tt)*],
        )
    }};

    // on invalid syntax
    (
        @go:
        $path:expr,
        [$($tt:tt)*],
    ) => {{
        compile_error!(concat!("Unexpected tokens ", stringify!($($tt)*)))
    }};

    // entry point
    ( $($tt:tt)* ) => {{
        $crate::key_path!(
            @go:
            $crate::key_path::KeyPath::default(),
            [$($tt)*],
        )
    }};
}

impl fmt::Display for KeyPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for key in &self.path {
            match key {
                Key::Field(field) => write!(f, ".{field}")?,
                Key::Element(field) => write!(f, "[{field}]")?,
                Key::Variant(variant) => write!(f, "{{{variant}}}")?,
            }
        }

        Ok(())
    }
}
