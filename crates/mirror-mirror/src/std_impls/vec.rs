use alloc::boxed::Box;
use alloc::vec::Vec;
use core::any::Any;

use crate::array::Array;
use crate::iter::ValueIterMut;
use crate::type_info::graph::Id;
use crate::type_info::graph::ListInfoNode;
use crate::type_info::graph::TypeInfoGraph;
use crate::FromReflect;
use crate::List;
use crate::Reflect;
use crate::ReflectMut;
use crate::ReflectRef;
use crate::TypeInfoRoot;
use crate::Typed;
use crate::Value;

impl<T> List for Vec<T>
where
    T: FromReflect + Typed,
{
    fn push(&mut self, value: &dyn Reflect) {
        if let Some(value) = T::from_reflect(value) {
            Vec::push(self, value);
        }
    }

    fn pop(&mut self) -> Option<Box<dyn Reflect>> {
        let value = Vec::pop(self)?;
        Some(Box::new(value))
    }

    fn try_remove(&mut self, index: usize) -> Option<Box<dyn Reflect>> {
        if index < self.len() {
            let value = Vec::remove(self, index);
            Some(Box::new(value))
        } else {
            None
        }
    }
}

impl<T> Array for Vec<T>
where
    T: FromReflect + Typed,
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

    fn iter(&self) -> crate::array::Iter<'_> {
        crate::array::Iter::new(self)
    }

    fn iter_mut(&mut self) -> ValueIterMut<'_> {
        let iter = self
            .as_mut_slice()
            .iter_mut()
            .map(|value| value.as_reflect_mut());
        Box::new(iter)
    }
}

impl<T> Reflect for Vec<T>
where
    T: FromReflect + Typed,
{
    fn type_info(&self) -> TypeInfoRoot {
        impl<T> Typed for Vec<T>
        where
            T: Typed,
        {
            fn build(graph: &mut TypeInfoGraph) -> Id {
                graph.get_or_build_with::<Self, _>(|graph| ListInfoNode::new::<Self, T>(graph))
            }
        }

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
        if let Some(list) = value.reflect_ref().as_list() {
            for (idx, new_value) in list.iter().enumerate() {
                if let Some(value) = self.get_mut(idx) {
                    value.patch(new_value);
                }
            }
        }
    }

    fn to_value(&self) -> Value {
        let data = self.iter().map(Reflect::to_value).collect();
        Value::List(data)
    }

    fn clone_reflect(&self) -> Box<dyn Reflect> {
        let value = self.to_value();
        Box::new(Self::from_reflect(&value).unwrap())
    }

    fn debug(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
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
    T: FromReflect + Typed,
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

impl<T> From<Vec<T>> for Value
where
    T: Reflect,
{
    fn from(list: Vec<T>) -> Self {
        let list = list
            .into_iter()
            .map(|value| value.to_value())
            .collect::<Vec<_>>();
        Value::List(list)
    }
}
