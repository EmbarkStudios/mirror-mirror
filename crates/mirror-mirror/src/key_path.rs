use std::fmt;

use serde::Deserialize;
use serde::Serialize;
use speedy::Readable;
use speedy::Writable;

use crate::Reflect;
use crate::ReflectMut;
use crate::ReflectRef;

pub trait GetPath {
    fn at(&self, key_path: KeyPath) -> Option<&dyn Reflect>;

    fn at_mut(&mut self, key_path: KeyPath) -> Option<&mut dyn Reflect>;

    fn get_at<T>(&self, key_path: KeyPath) -> Option<&T>
    where
        T: Reflect,
    {
        self.at(key_path)?.downcast_ref()
    }

    fn get_at_mut<T>(&mut self, key_path: KeyPath) -> Option<&mut T>
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
    fn at(&self, key_path: KeyPath) -> Option<&dyn Reflect> {
        fn go<R>(value: &R, mut stack: Vec<Key>) -> Option<&dyn Reflect>
        where
            R: Reflect + ?Sized,
        {
            let head = stack.pop()?;

            let value_at_key = match head {
                Key::Field(key) => {
                    let key: &str = &key;
                    match value.reflect_ref() {
                        ReflectRef::Struct(inner) => inner.field(key)?,
                        ReflectRef::Map(inner) => inner.get(&key.to_owned())?,
                        ReflectRef::Enum(inner) => inner.field(key)?,
                        ReflectRef::TupleStruct(_)
                        | ReflectRef::Tuple(_)
                        | ReflectRef::List(_)
                        | ReflectRef::Scalar(_) => return None,
                    }
                }
                Key::Element(index) => match value.reflect_ref() {
                    ReflectRef::TupleStruct(inner) => inner.element(index)?,
                    ReflectRef::Tuple(inner) => inner.element(index)?,
                    ReflectRef::Enum(inner) => inner.element(index)?,
                    ReflectRef::List(inner) => inner.get(index)?,
                    ReflectRef::Map(inner) => inner.get(&index)?,
                    ReflectRef::Struct(_) | ReflectRef::Scalar(_) => return None,
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
                    | ReflectRef::Map(_)
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

        let mut path = key_path.path;
        path.reverse();
        go(self, path)
    }

    fn at_mut(&mut self, key_path: KeyPath) -> Option<&mut dyn Reflect> {
        fn go<R>(value: &mut R, mut stack: Vec<Key>) -> Option<&mut dyn Reflect>
        where
            R: Reflect + ?Sized,
        {
            let head = stack.pop()?;

            let value_at_key = match head {
                Key::Field(key) => match value.reflect_mut() {
                    ReflectMut::Struct(inner) => inner.field_mut(&key)?,
                    ReflectMut::Map(inner) => inner.get_mut(&key)?,
                    ReflectMut::Enum(inner) => inner.field_mut(&key)?,
                    ReflectMut::TupleStruct(_)
                    | ReflectMut::Tuple(_)
                    | ReflectMut::List(_)
                    | ReflectMut::Scalar(_) => return None,
                },
                Key::Element(index) => match value.reflect_mut() {
                    ReflectMut::TupleStruct(inner) => inner.element_mut(index)?,
                    ReflectMut::Tuple(inner) => inner.element_mut(index)?,
                    ReflectMut::Enum(inner) => inner.element_mut(index)?,
                    ReflectMut::List(inner) => inner.get_mut(index)?,
                    ReflectMut::Map(inner) => inner.get_mut(&index)?,
                    ReflectMut::Struct(_) | ReflectMut::Scalar(_) => return None,
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
                    | ReflectMut::Map(_)
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

        let mut path = key_path.path;
        path.reverse();
        go(self, path)
    }
}

#[derive(Readable, Writable, Serialize, Deserialize, Debug, Clone, Default)]
pub struct KeyPath {
    pub(crate) path: Vec<Key>,
}

impl KeyPath {
    pub fn field<S>(mut self, field: S) -> Self
    where
        S: Into<String>,
    {
        self.push_field(field);
        self
    }

    pub fn push_field<S>(&mut self, field: S)
    where
        S: Into<String>,
    {
        self.path.push(Key::Field(field.into()));
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

    pub fn element(mut self, element: usize) -> Self {
        self.push_element(element);
        self
    }

    pub fn push_element(&mut self, element: usize) {
        self.path.push(Key::Element(element));
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

#[derive(Readable, Writable, Serialize, Deserialize, Debug, Clone)]
pub(crate) enum Key {
    Field(String),
    Element(usize),
    Variant(String),
}

pub fn field<S>(field: S) -> KeyPath
where
    S: Into<String>,
{
    KeyPath::default().field(field)
}

pub fn element(element: usize) -> KeyPath {
    KeyPath::default().element(element)
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

    // recursive case (element)
    (
        @go:
        $path:expr,
        [ [$element:expr] $($tt:tt)*],
    ) => {{
        $crate::key_path!(
            @go:
            $path.element($element),
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
                Key::Element(element) => write!(f, "[{element}]")?,
                Key::Variant(variant) => write!(f, "{{{variant}}}")?,
            }
        }

        Ok(())
    }
}
