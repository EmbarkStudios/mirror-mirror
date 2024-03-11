use alloc::boxed::Box;
use alloc::vec::Vec;
use core::any::Any;

use crate::array::Array;
use crate::iter::ValueIterMut;
use crate::list::ListError;
use crate::type_info::graph::ListNode;
use crate::type_info::graph::NodeId;
use crate::type_info::graph::TypeGraph;
use crate::DescribeType;
use crate::FromReflect;
use crate::List;
use crate::Reflect;
use crate::ReflectMut;
use crate::ReflectOwned;
use crate::ReflectRef;
use crate::Value;

impl<T> List for Vec<T>
where
    T: FromReflect + DescribeType,
{
    fn try_push(&mut self, element: &dyn Reflect) -> Result<(), ListError> {
        if let Some(value) = T::from_reflect(element) {
            Vec::push(self, value);
            Ok(())
        } else {
            Err(ListError)
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

    fn try_insert(&mut self, index: usize, element: &dyn Reflect) -> Result<(), ListError> {
        if let Some(element) = T::from_reflect(element) {
            Vec::insert(self, index, element);
            Ok(())
        } else {
            Err(ListError)
        }
    }
}

impl<T> Array for Vec<T>
where
    T: FromReflect + DescribeType,
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

    fn swap(&mut self, a: usize, b: usize) {
        self.as_mut_slice().swap(a, b);
    }
}

impl<T> DescribeType for Vec<T>
where
    T: DescribeType,
{
    fn build(graph: &mut TypeGraph) -> NodeId {
        graph.get_or_build_node_with::<Self, _>(|graph| ListNode::new::<Self, T>(graph))
    }
}

impl<T> Reflect for Vec<T>
where
    T: FromReflect + DescribeType,
{
    trivial_reflect_methods!();

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

    fn reflect_owned(self: Box<Self>) -> ReflectOwned {
        ReflectOwned::List(self)
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
    T: FromReflect + DescribeType,
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
