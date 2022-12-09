use alloc::boxed::Box;
use core::any::Any;
use core::fmt;

use crate::iter::ValueIterMut;
use crate::tuple::TupleValue;
use crate::type_info::graph::NodeId;
use crate::type_info::graph::OpaqueNode;
use crate::type_info::graph::TypeGraph;
use crate::FromReflect;
use crate::Reflect;
use crate::ReflectMut;
use crate::ReflectOwned;
use crate::ReflectRef;
use crate::Tuple;
use crate::TypeRoot;
use crate::Typed;
use crate::Value;

pub trait TupleStruct: Reflect {
    fn field_at(&self, index: usize) -> Option<&dyn Reflect>;

    fn field_at_mut(&mut self, index: usize) -> Option<&mut dyn Reflect>;

    fn fields(&self) -> Iter<'_>;

    fn fields_mut(&mut self) -> ValueIterMut<'_>;

    fn fields_len(&self) -> usize;
}

impl fmt::Debug for dyn TupleStruct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_reflect().debug(f)
    }
}

#[derive(Default, Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "speedy", derive(speedy::Readable, speedy::Writable))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TupleStructValue {
    tuple: TupleValue,
}

impl TupleStructValue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_field(self, value: impl Into<Value>) -> Self {
        Self {
            tuple: self.tuple.with_field(value),
        }
    }

    pub fn push_field(&mut self, value: impl Into<Value>) {
        self.tuple.push_field(value);
    }
}

impl Reflect for TupleStructValue {
    fn type_info(&self) -> TypeRoot {
        impl Typed for TupleStructValue {
            fn build(graph: &mut TypeGraph) -> NodeId {
                graph.get_or_build_node_with::<Self, _>(|graph| {
                    OpaqueNode::new::<Self>(Default::default(), graph)
                })
            }
        }
        <Self as Typed>::type_info()
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
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
        if let Some(tuple) = value.reflect_ref().as_tuple_struct() {
            for (index, value) in self.fields_mut().enumerate() {
                if let Some(new_value) = tuple.field_at(index) {
                    value.patch(new_value);
                }
            }
        }
    }

    fn to_value(&self) -> Value {
        self.clone().into()
    }

    fn clone_reflect(&self) -> Box<dyn Reflect> {
        Box::new(self.clone())
    }

    fn debug(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if f.alternate() {
            write!(f, "{:#?}", self)
        } else {
            write!(f, "{:?}", self)
        }
    }

    fn reflect_owned(self: Box<Self>) -> ReflectOwned {
        ReflectOwned::TupleStruct(self)
    }

    fn reflect_ref(&self) -> ReflectRef<'_> {
        ReflectRef::TupleStruct(self)
    }

    fn reflect_mut(&mut self) -> ReflectMut<'_> {
        ReflectMut::TupleStruct(self)
    }
}

impl TupleStruct for TupleStructValue {
    fn field_at(&self, index: usize) -> Option<&dyn Reflect> {
        self.tuple.field_at(index)
    }

    fn field_at_mut(&mut self, index: usize) -> Option<&mut dyn Reflect> {
        self.tuple.field_at_mut(index)
    }

    fn fields(&self) -> Iter<'_> {
        Iter::new(self)
    }

    fn fields_mut(&mut self) -> ValueIterMut<'_> {
        self.tuple.fields_mut()
    }

    fn fields_len(&self) -> usize {
        self.tuple.fields_len()
    }
}

impl FromReflect for TupleStructValue {
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
        let tuple_struct = reflect.reflect_ref().as_tuple_struct()?;
        let this = tuple_struct
            .fields()
            .fold(TupleStructValue::default(), |builder, value| {
                builder.with_field(value.to_value())
            });
        Some(this)
    }
}

impl<V> FromIterator<V> for TupleStructValue
where
    V: Reflect,
{
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = V>,
    {
        let mut out = Self::default();
        for value in iter {
            out.push_field(value.to_value());
        }
        out
    }
}

#[derive(Debug)]
pub struct Iter<'a> {
    tuple_struct: &'a dyn TupleStruct,
    index: usize,
}

impl<'a> Iter<'a> {
    pub fn new(tuple_struct: &'a dyn TupleStruct) -> Self {
        Self {
            tuple_struct,
            index: 0,
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a dyn Reflect;

    fn next(&mut self) -> Option<Self::Item> {
        let value = self.tuple_struct.field_at(self.index)?;
        self.index += 1;
        Some(value)
    }
}
