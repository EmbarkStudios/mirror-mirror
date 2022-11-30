use crate::Enum;
use crate::List;
use crate::Map;
use crate::Reflect;
use crate::ReflectMut;
use crate::ReflectRef;
use crate::Struct;
use crate::Tuple;
use crate::TupleStruct;
use crate::Value;

pub trait GetField<'a, K, M> {
    fn get_field<T>(self, key: K) -> Option<&'a T>
    where
        T: Reflect;
}

pub trait GetFieldMut<'a, K, M> {
    fn get_field_mut<T>(self, key: K) -> Option<&'a mut T>
    where
        T: Reflect;
}

impl<'a, R, K, M> GetField<'a, K, M> for &'a mut R
where
    R: ?Sized,
    &'a R: GetField<'a, K, M>,
{
    fn get_field<T>(self, key: K) -> Option<&'a T>
    where
        T: Reflect,
    {
        <&R as GetField<_, _>>::get_field(self, key)
    }
}

impl<'a> GetField<'a, &str, private::Value> for &'a Value {
    #[allow(warnings)]
    fn get_field<T>(self, key: &str) -> Option<&'a T>
    where
        T: Reflect,
    {
        match self.reflect_ref() {
            ReflectRef::Struct(inner) => inner.get_field(key),
            ReflectRef::Enum(inner) => inner.get_field(key),
            ReflectRef::Map(inner) => inner.get_field(key),
            ReflectRef::TupleStruct(_)
            | ReflectRef::Tuple(_)
            | ReflectRef::List(_)
            | ReflectRef::Scalar(_) => None,
        }
    }
}

impl<'a> GetFieldMut<'a, &str, private::Value> for &'a mut Value {
    fn get_field_mut<T>(self, key: &str) -> Option<&'a mut T>
    where
        T: Reflect,
    {
        match self.reflect_mut() {
            ReflectMut::Struct(inner) => inner.get_field_mut(key),
            ReflectMut::Enum(inner) => inner.get_field_mut(key),
            ReflectMut::Map(inner) => inner.get_field_mut(key),
            ReflectMut::TupleStruct(_)
            | ReflectMut::Tuple(_)
            | ReflectMut::List(_)
            | ReflectMut::Scalar(_) => None,
        }
    }
}

impl<'a, K> GetField<'a, K, private::Value> for &'a Value
where
    K: Reflect,
{
    fn get_field<T>(self, key: K) -> Option<&'a T>
    where
        T: Reflect,
    {
        if let Some(&key) = key.as_any().downcast_ref::<usize>() {
            match self.reflect_ref() {
                ReflectRef::TupleStruct(inner) => inner.get_field(key),
                ReflectRef::Tuple(inner) => inner.get_field(key),
                ReflectRef::Enum(inner) => inner.get_field(key),
                ReflectRef::List(inner) => inner.get_field(key),
                ReflectRef::Map(inner) => inner.get_field(key),
                ReflectRef::Struct(_) | ReflectRef::Scalar(_) => None,
            }
        } else if let Some(key) = key.as_any().downcast_ref::<String>() {
            match self.reflect_ref() {
                ReflectRef::Map(inner) => inner.get_field(key.to_owned()),
                ReflectRef::Struct(inner) => inner.get_field(key),
                ReflectRef::TupleStruct(_)
                | ReflectRef::Tuple(_)
                | ReflectRef::Enum(_)
                | ReflectRef::List(_)
                | ReflectRef::Scalar(_) => None,
            }
        } else {
            match self.reflect_ref() {
                ReflectRef::Map(inner) => inner.get_field(key),
                ReflectRef::TupleStruct(_)
                | ReflectRef::Tuple(_)
                | ReflectRef::Enum(_)
                | ReflectRef::List(_)
                | ReflectRef::Struct(_)
                | ReflectRef::Scalar(_) => None,
            }
        }
    }
}

impl<'a, K> GetFieldMut<'a, K, private::Value> for &'a mut Value
where
    K: Reflect,
{
    fn get_field_mut<T>(self, key: K) -> Option<&'a mut T>
    where
        T: Reflect,
    {
        if let Some(&key) = key.as_any().downcast_ref::<usize>() {
            match self.reflect_mut() {
                ReflectMut::TupleStruct(inner) => inner.get_field_mut(key),
                ReflectMut::Tuple(inner) => inner.get_field_mut(key),
                ReflectMut::Enum(inner) => inner.get_field_mut(key),
                ReflectMut::List(inner) => inner.get_field_mut(key),
                ReflectMut::Map(inner) => inner.get_field_mut(key),
                ReflectMut::Struct(_) | ReflectMut::Scalar(_) => None,
            }
        } else if let Some(key) = key.as_any().downcast_ref::<String>() {
            match self.reflect_mut() {
                ReflectMut::Map(inner) => inner.get_field_mut(key.to_owned()),
                ReflectMut::Struct(inner) => inner.get_field_mut(key),
                ReflectMut::TupleStruct(_)
                | ReflectMut::Tuple(_)
                | ReflectMut::Enum(_)
                | ReflectMut::List(_)
                | ReflectMut::Scalar(_) => None,
            }
        } else {
            match self.reflect_mut() {
                ReflectMut::Map(inner) => inner.get_field_mut(key),
                ReflectMut::TupleStruct(_)
                | ReflectMut::Tuple(_)
                | ReflectMut::Enum(_)
                | ReflectMut::List(_)
                | ReflectMut::Struct(_)
                | ReflectMut::Scalar(_) => None,
            }
        }
    }
}

impl<'a, R> GetField<'a, &str, private::Struct> for &'a R
where
    R: Struct + ?Sized,
{
    fn get_field<T>(self, key: &str) -> Option<&'a T>
    where
        T: Reflect,
    {
        self.field(key)?.downcast_ref()
    }
}

impl<'a, R> GetFieldMut<'a, &str, private::Struct> for &'a mut R
where
    R: Struct + ?Sized,
{
    fn get_field_mut<T>(self, key: &str) -> Option<&'a mut T>
    where
        T: Reflect,
    {
        self.field_mut(key)?.downcast_mut()
    }
}

impl<'a, R> GetField<'a, usize, private::TupleStruct> for &'a R
where
    R: TupleStruct + ?Sized,
{
    fn get_field<T>(self, key: usize) -> Option<&'a T>
    where
        T: Reflect,
    {
        self.element(key)?.downcast_ref()
    }
}

impl<'a, R> GetFieldMut<'a, usize, private::TupleStruct> for &'a mut R
where
    R: TupleStruct + ?Sized,
{
    fn get_field_mut<T>(self, key: usize) -> Option<&'a mut T>
    where
        T: Reflect,
    {
        self.element_mut(key)?.downcast_mut()
    }
}

impl<'a, R> GetField<'a, &str, private::Enum> for &'a R
where
    R: Enum + ?Sized,
{
    fn get_field<T>(self, key: &str) -> Option<&'a T>
    where
        T: Reflect,
    {
        self.field(key)?.downcast_ref()
    }
}

impl<'a, R> GetFieldMut<'a, &str, private::Enum> for &'a mut R
where
    R: Enum + ?Sized,
{
    fn get_field_mut<T>(self, key: &str) -> Option<&'a mut T>
    where
        T: Reflect,
    {
        self.field_mut(key)?.downcast_mut()
    }
}

impl<'a, R> GetField<'a, usize, private::Enum> for &'a R
where
    R: Enum + ?Sized,
{
    fn get_field<T>(self, key: usize) -> Option<&'a T>
    where
        T: Reflect,
    {
        self.element(key)?.downcast_ref()
    }
}

impl<'a, R> GetFieldMut<'a, usize, private::Enum> for &'a mut R
where
    R: Enum + ?Sized,
{
    fn get_field_mut<T>(self, key: usize) -> Option<&'a mut T>
    where
        T: Reflect,
    {
        self.element_mut(key)?.downcast_mut()
    }
}

impl<'a, R> GetField<'a, usize, private::Tuple> for &'a R
where
    R: Tuple + ?Sized,
{
    fn get_field<T>(self, key: usize) -> Option<&'a T>
    where
        T: Reflect,
    {
        self.element(key)?.downcast_ref()
    }
}

impl<'a, R> GetFieldMut<'a, usize, private::Tuple> for &'a mut R
where
    R: Tuple + ?Sized,
{
    fn get_field_mut<T>(self, key: usize) -> Option<&'a mut T>
    where
        T: Reflect,
    {
        self.element_mut(key)?.downcast_mut()
    }
}

impl<'a, R> GetField<'a, usize, private::List> for &'a R
where
    R: List + ?Sized,
{
    fn get_field<T>(self, key: usize) -> Option<&'a T>
    where
        T: Reflect,
    {
        self.get(key)?.downcast_ref()
    }
}

impl<'a, R> GetFieldMut<'a, usize, private::List> for &'a mut R
where
    R: List + ?Sized,
{
    fn get_field_mut<T>(self, key: usize) -> Option<&'a mut T>
    where
        T: Reflect,
    {
        self.get_mut(key)?.downcast_mut()
    }
}

impl<'a, R, K> GetField<'a, K, private::Map> for &'a R
where
    R: Map + ?Sized,
    K: Reflect,
{
    fn get_field<T>(self, key: K) -> Option<&'a T>
    where
        T: Reflect,
    {
        self.get(&key)?.downcast_ref()
    }
}

impl<'a, R, K> GetFieldMut<'a, K, private::Map> for &'a mut R
where
    R: Map + ?Sized,
    K: Reflect,
{
    fn get_field_mut<T>(self, key: K) -> Option<&'a mut T>
    where
        T: Reflect,
    {
        self.get_mut(&key)?.downcast_mut()
    }
}

impl<'a, R> GetField<'a, &str, private::Map> for &'a R
where
    R: Map + ?Sized,
{
    fn get_field<T>(self, key: &str) -> Option<&'a T>
    where
        T: Reflect,
    {
        self.get(&key.to_owned())?.downcast_ref()
    }
}

impl<'a, R> GetFieldMut<'a, &str, private::Map> for &'a mut R
where
    R: Map + ?Sized,
{
    fn get_field_mut<T>(self, key: &str) -> Option<&'a mut T>
    where
        T: Reflect,
    {
        self.get_mut(&key.to_owned())?.downcast_mut()
    }
}

mod private {
    /// Types used to disambiguate otherwise overlapping trait impls

    pub struct Struct;
    pub struct TupleStruct;
    pub struct Enum;
    pub struct Tuple;
    pub struct List;
    pub struct Map;
    pub struct Value;
}
