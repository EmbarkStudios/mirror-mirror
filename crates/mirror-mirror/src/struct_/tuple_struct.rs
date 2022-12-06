use crate::iter::ValueIter;
use crate::iter::ValueIterMut;
use crate::tuple::TupleValue;
use crate::type_info::graph::Id;
use crate::type_info::graph::OpaqueInfoNode;
use crate::type_info::graph::TypeInfoGraph;
use crate::FromReflect;
use crate::Reflect;
use crate::ReflectMut;
use crate::ReflectRef;
use crate::Tuple;
use crate::TypeInfoRoot;
use crate::Typed;
use crate::Value;
use serde::Deserialize;
use serde::Serialize;
use std::any::Any;
use std::fmt;

pub trait TupleStruct: Reflect {
    fn field(&self, index: usize) -> Option<&dyn Reflect>;

    fn field_mut(&mut self, index: usize) -> Option<&mut dyn Reflect>;

    fn fields(&self) -> ValueIter<'_>;

    fn fields_mut(&mut self) -> ValueIterMut<'_>;
}

impl fmt::Debug for dyn TupleStruct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_reflect().debug(f)
    }
}

#[derive(Default, Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "speedy", derive(speedy::Readable, speedy::Writable))]
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
    fn type_info(&self) -> TypeInfoRoot {
        impl Typed for TupleStructValue {
            fn build(graph: &mut TypeInfoGraph) -> Id {
                graph.get_or_build_with::<Self, _>(|graph| {
                    OpaqueInfoNode::new::<Self>(Default::default(), graph)
                })
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
        if let Some(tuple) = value.reflect_ref().as_tuple_struct() {
            for (index, value) in self.fields_mut().enumerate() {
                if let Some(new_value) = tuple.field(index) {
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

    fn debug(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            write!(f, "{:#?}", self)
        } else {
            write!(f, "{:?}", self)
        }
    }

    fn reflect_ref(&self) -> ReflectRef<'_> {
        ReflectRef::TupleStruct(self)
    }

    fn reflect_mut(&mut self) -> ReflectMut<'_> {
        ReflectMut::TupleStruct(self)
    }
}

impl TupleStruct for TupleStructValue {
    fn field(&self, index: usize) -> Option<&dyn Reflect> {
        self.tuple.field(index)
    }

    fn field_mut(&mut self, index: usize) -> Option<&mut dyn Reflect> {
        self.tuple.field_mut(index)
    }

    fn fields(&self) -> ValueIter<'_> {
        self.tuple.fields()
    }

    fn fields_mut(&mut self) -> ValueIterMut<'_> {
        self.tuple.fields_mut()
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
