use alloc::borrow::ToOwned;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;
use core::iter::FusedIterator;
use core::iter::Peekable;

use crate::enum_::VariantKind;
use crate::type_info::TypeAtPath;
use crate::Reflect;
use crate::ReflectMut;
use crate::ReflectRef;
use crate::Value;

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
    fn type_at(self, key_path: &KeyPath) -> Option<TypeAtPath<'a>>;
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
                Key::Field(KeyOrIndex::Key(key)) => match value.reflect_ref() {
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
                Key::Field(KeyOrIndex::Index(index)) => match value.reflect_ref() {
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
                // ["foo"] or [0]
                Key::Get(key) => match value.reflect_ref() {
                    ReflectRef::Map(inner) => inner.get(key)?,
                    ReflectRef::Array(inner) => inner.get(value_to_usize(key)?)?,
                    ReflectRef::List(inner) => inner.get(value_to_usize(key)?)?,
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
                Key::Field(KeyOrIndex::Key(key)) => match value.reflect_mut() {
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
                Key::Field(KeyOrIndex::Index(index)) => match value.reflect_mut() {
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
                // ["foo"] or [0]
                Key::Get(key) => match value.reflect_mut() {
                    ReflectMut::Array(inner) => inner.get_mut(value_to_usize(key)?)?,
                    ReflectMut::List(inner) => inner.get_mut(value_to_usize(key)?)?,
                    ReflectMut::Map(inner) => inner.get_mut(key)?,
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
        self.push(Key::Field(field.into_key_or_index()));
    }

    pub fn get(mut self, field: impl Into<Value>) -> Self {
        self.push_get(field);
        self
    }

    pub fn push_get(&mut self, field: impl Into<Value>) {
        self.push(Key::Get(field.into()))
    }

    pub fn variant(mut self, variant: impl Into<String>) -> Self {
        self.push_variant(variant);
        self
    }

    pub fn push_variant(&mut self, variant: impl Into<String>) {
        self.push(Key::Variant(variant.into()));
    }

    pub fn push(&mut self, key: Key) {
        self.path.push(key);
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

    pub fn iter(&self) -> Iter<'_> {
        self.into_iter()
    }
}

impl IntoIterator for KeyPath {
    type Item = Key;
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self.path.into_iter())
    }
}

#[derive(Debug)]
pub struct IntoIter(alloc::vec::IntoIter<Key>);

impl Iterator for IntoIter {
    type Item = Key;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl DoubleEndedIterator for IntoIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}

impl ExactSizeIterator for IntoIter {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl FusedIterator for IntoIter {}

impl<'a> IntoIterator for &'a KeyPath {
    type Item = &'a Key;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        Iter(self.path.iter())
    }
}

#[derive(Debug)]
pub struct Iter<'a>(alloc::slice::Iter<'a, Key>);

impl<'a> Iterator for Iter<'a> {
    type Item = &'a Key;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<'a> DoubleEndedIterator for Iter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}

impl<'a> ExactSizeIterator for Iter<'a> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a> FusedIterator for Iter<'a> {}

mod private {
    use super::*;

    pub trait Sealed {}
    impl Sealed for &str {}
    impl Sealed for &String {}
    impl Sealed for String {}
    impl Sealed for usize {}
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "speedy", derive(speedy::Readable, speedy::Writable))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Key {
    Field(KeyOrIndex),
    Get(Value),
    Variant(String),
}

impl Key {
    /// Create a `Key` that'll access a named field of a struct.
    pub fn field(name: impl Into<String>) -> Self {
        Self::Field(KeyOrIndex::Key(name.into()))
    }

    /// Create a `Key` that'll access a numbered field of a tuple struct or tuple.
    pub fn field_at(index: usize) -> Self {
        Self::Field(KeyOrIndex::Index(index))
    }

    /// Create a `Key` that'll access an element in a list, array, or map.
    pub fn get(value: impl Into<Value>) -> Self {
        Self::Get(value.into())
    }

    /// Create a `Key` that'll access an enum variant.
    pub fn variant(name: impl Into<String>) -> Self {
        Self::Variant(name.into())
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Key::Field(key) => write!(f, "{key}"),
            Key::Get(value) => write!(f, "[{:?}]", value.as_reflect()),
            Key::Variant(variant) => write!(f, "::{variant}"),
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "speedy", derive(speedy::Readable, speedy::Writable))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum KeyOrIndex {
    Key(String),
    Index(usize),
}

impl fmt::Display for KeyOrIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeyOrIndex::Key(key) => write!(f, ".{key}"),
            KeyOrIndex::Index(index) => write!(f, ".{index}"),
        }
    }
}

pub trait IntoKeyOrIndex: private::Sealed {
    fn into_key_or_index(self) -> KeyOrIndex;
}

impl IntoKeyOrIndex for &str {
    fn into_key_or_index(self) -> KeyOrIndex {
        KeyOrIndex::Key(self.to_owned())
    }
}

impl IntoKeyOrIndex for &String {
    fn into_key_or_index(self) -> KeyOrIndex {
        KeyOrIndex::Key(self.to_owned())
    }
}

impl IntoKeyOrIndex for String {
    fn into_key_or_index(self) -> KeyOrIndex {
        KeyOrIndex::Key(self)
    }
}

impl IntoKeyOrIndex for usize {
    fn into_key_or_index(self) -> KeyOrIndex {
        KeyOrIndex::Index(self)
    }
}

pub fn field(field: impl IntoKeyOrIndex) -> KeyPath {
    KeyPath::default().field(field)
}

pub fn get(field: impl Into<Value>) -> KeyPath {
    KeyPath::default().get(field)
}

pub fn variant(variant: impl Into<String>) -> KeyPath {
    KeyPath::default().variant(variant)
}

/// Convenience macro for creating [`KeyPath`]s.
///
/// Expands to calls to [`field`], [`get`], [`variant`], and methods on [`KeyPath`].
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
            write!(f, "{key}")?;
        }
        Ok(())
    }
}

pub(crate) fn value_to_usize(value: &Value) -> Option<usize> {
    match value {
        Value::usize(n) => Some(*n),
        Value::u8(n) => Some(*n as usize),
        Value::u16(n) => Some(*n as usize),
        Value::u32(n) => Some(*n as usize),
        Value::u64(n) => Some(*n as usize),
        Value::u128(n) => Some(*n as usize),
        Value::i8(n) => Some(*n as usize),
        Value::i16(n) => Some(*n as usize),
        Value::i32(n) => Some(*n as usize),
        Value::i64(n) => Some(*n as usize),
        Value::i128(n) => Some(*n as usize),
        Value::bool(_)
        | Value::char(_)
        | Value::f32(_)
        | Value::f64(_)
        | Value::String(_)
        | Value::StructValue(_)
        | Value::EnumValue(_)
        | Value::TupleStructValue(_)
        | Value::TupleValue(_)
        | Value::List(_)
        | Value::Map(_) => None,
    }
}
