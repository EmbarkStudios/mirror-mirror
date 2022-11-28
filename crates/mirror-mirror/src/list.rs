use crate::FromReflect;
use crate::Reflect;
use crate::ReflectMut;
use crate::ReflectRef;
use crate::Value;
use crate::ValueData;
use crate::ValueIter;
use crate::ValueIterMut;
use std::fmt;

pub trait List: Reflect {
    fn get(&self, index: usize) -> Option<&dyn Reflect>;
    fn get_mut(&mut self, index: usize) -> Option<&mut dyn Reflect>;

    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;

    fn iter(&self) -> ValueIter<'_>;
    fn iter_mut(&mut self) -> ValueIterMut<'_>;
}

impl fmt::Debug for dyn List {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_reflect().debug(f)
    }
}

impl<T> List for Vec<T>
where
    T: FromReflect,
{
    fn get(&self, index: usize) -> Option<&dyn Reflect> {
        self.as_slice().get(index).map(|value| value.as_reflect())
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut dyn Reflect> {
        self.as_mut_slice()
            .get_mut(index)
            .map(|value| value.as_reflect_mut())
    }

    fn len(&self) -> usize {
        Vec::len(self)
    }

    fn is_empty(&self) -> bool {
        Vec::is_empty(self)
    }

    fn iter(&self) -> ValueIter<'_> {
        let iter = self.as_slice().iter().map(|value| value.as_reflect());
        ValueIter::new(iter)
    }

    fn iter_mut(&mut self) -> ValueIterMut<'_> {
        let iter = self
            .as_mut_slice()
            .iter_mut()
            .map(|value| value.as_reflect_mut());
        ValueIterMut::new(iter)
    }
}

impl<T> Reflect for Vec<T>
where
    T: FromReflect,
{
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn as_reflect(&self) -> &dyn Reflect {
        self
    }

    fn as_reflect_mut(&mut self) -> &mut dyn Reflect {
        self
    }

    fn patch(&mut self, value: &dyn Reflect) {
        if let Some(list) = value.reflect_ref().as_list() {
            for (idx, new_value) in list.iter().enumerate() {
                if let Some(value) = self.get_mut(idx) {
                    value.patch(new_value);
                }
            }
        }
    }

    fn to_value(&self) -> Value {
        let data = self.iter().map(Reflect::to_value).collect::<Vec<_>>();
        Value::new(ValueData::List(data))
    }

    fn clone_reflect(&self) -> Box<dyn Reflect> {
        let value = self.to_value();
        Box::new(Self::from_reflect(&value).unwrap())
    }

    fn debug(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }

    fn reflect_ref(&self) -> ReflectRef<'_> {
        ReflectRef::List(self)
    }

    fn reflect_mut(&mut self) -> ReflectMut<'_> {
        ReflectMut::List(self)
    }
}

impl<T> FromReflect for Vec<T>
where
    T: FromReflect,
{
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
        let list = reflect.reflect_ref().as_list()?;
        let mut out = Vec::new();
        for value in list.iter() {
            out.push(T::from_reflect(value)?);
        }
        Some(out)
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn indexing() {
        let list = Vec::from([1, 2, 3]);
        let list = list.reflect_ref().as_list().unwrap();

        assert_eq!(list.get(0).unwrap().downcast_ref::<i32>().unwrap(), &1);
        assert_eq!(list.get(1).unwrap().downcast_ref::<i32>().unwrap(), &2);
        assert_eq!(list.get(2).unwrap().downcast_ref::<i32>().unwrap(), &3);
        assert!(list.get(3).is_none());

        let value = list.to_value();
        let value = value.reflect_ref().as_list().unwrap();
        assert_eq!(value.get(0).unwrap().downcast_ref::<i32>().unwrap(), &1);
        assert_eq!(value.get(1).unwrap().downcast_ref::<i32>().unwrap(), &2);
        assert_eq!(value.get(2).unwrap().downcast_ref::<i32>().unwrap(), &3);
        assert!(value.get(3).is_none());

        let mut list = Vec::<i32>::from_reflect(list.as_reflect()).unwrap();
        assert_eq!(list, Vec::from([1, 2, 3]));

        list.patch(&Vec::from([42]));
        assert_eq!(list, Vec::from([42, 2, 3]));
    }

    #[test]
    fn debug() {
        let list = Vec::from([1, 2, 3]);
        assert_eq!(format!("{:?}", list.as_reflect()), format!("{:?}", list));
        assert_eq!(format!("{:#?}", list.as_reflect()), format!("{:#?}", list));
    }
}
