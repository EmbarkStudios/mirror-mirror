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
    path: Vec<Key>,
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
enum Key {
    Field(String),
    Element(usize),
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
            }
        }

        Ok(())
    }
}
