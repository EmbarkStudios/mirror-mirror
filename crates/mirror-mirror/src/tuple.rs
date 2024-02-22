use alloc::boxed::Box;
use alloc::vec::Vec;
use core::any::Any;
use core::fmt;
use core::fmt::Debug;
use core::iter::FusedIterator;

use crate::iter::ValueIterMut;
use crate::type_info::graph::NodeId;
use crate::type_info::graph::OpaqueNode;
use crate::type_info::graph::TupleNode;
use crate::type_info::graph::TypeGraph;
use crate::type_info::graph::UnnamedFieldNode;
use crate::DescribeType;
use crate::FromReflect;
use crate::Reflect;
use crate::ReflectMut;
use crate::ReflectOwned;
use crate::ReflectRef;
use crate::Value;

/// A reflected tuple type.
pub trait Tuple: Reflect {
    fn field_at(&self, index: usize) -> Option<&dyn Reflect>;

    fn field_at_mut(&mut self, index: usize) -> Option<&mut dyn Reflect>;

    fn fields(&self) -> Iter<'_>;

    fn fields_mut(&mut self) -> ValueIterMut<'_>;

    fn fields_len(&self) -> usize;
}

impl fmt::Debug for dyn Tuple {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_reflect().debug(f)
    }
}

#[derive(Default, Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[cfg_attr(feature = "speedy", derive(speedy::Readable, speedy::Writable))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TupleValue {
    fields: Vec<Value>,
}

impl TupleValue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            fields: Vec::with_capacity(capacity),
        }
    }

    pub fn with_field(mut self, value: impl Into<Value>) -> Self {
        self.push_field(value);
        self
    }

    pub fn push_field(&mut self, value: impl Into<Value>) {
        self.fields.push(value.into());
    }
}

impl Tuple for TupleValue {
    fn field_at(&self, index: usize) -> Option<&dyn Reflect> {
        Some(self.fields.get(index)?.as_reflect())
    }

    fn field_at_mut(&mut self, index: usize) -> Option<&mut dyn Reflect> {
        Some(self.fields.get_mut(index)?.as_reflect_mut())
    }

    fn fields(&self) -> Iter<'_> {
        Iter::new(self)
    }

    fn fields_mut(&mut self) -> ValueIterMut<'_> {
        let iter = self.fields.iter_mut().map(|value| value.as_reflect_mut());
        Box::new(iter)
    }

    fn fields_len(&self) -> usize {
        self.fields.len()
    }
}

impl DescribeType for TupleValue {
    fn build(graph: &mut TypeGraph) -> NodeId {
        graph.get_or_build_node_with::<Self, _>(|graph| {
            OpaqueNode::new::<Self>(Default::default(), graph)
        })
    }
}

impl Reflect for TupleValue {
    trivial_reflect_methods!();

    fn patch(&mut self, value: &dyn Reflect) {
        if let Some(tuple) = value.reflect_ref().as_tuple() {
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
            write!(f, "{self:#?}")
        } else {
            write!(f, "{self:?}")
        }
    }

    fn reflect_owned(self: Box<Self>) -> ReflectOwned {
        ReflectOwned::Tuple(self)
    }

    fn reflect_ref(&self) -> ReflectRef<'_> {
        ReflectRef::Tuple(self)
    }

    fn reflect_mut(&mut self) -> ReflectMut<'_> {
        ReflectMut::Tuple(self)
    }
}

impl FromReflect for TupleValue {
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
        let tuple = reflect.reflect_ref().as_tuple()?;
        let this = tuple
            .fields()
            .fold(TupleValue::default(), |builder, value| {
                builder.with_field(value.to_value())
            });
        Some(this)
    }
}

macro_rules! impl_tuple {
    ($($ident:ident),* $(,)?) => {
        #[allow(non_snake_case, unused_mut, unused_variables)]
        impl<$($ident,)*> DescribeType for ($($ident,)*)
        where
            $($ident: Reflect + DescribeType + Clone,)*
        {
            fn build(graph: &mut TypeGraph) -> NodeId {
                graph.get_or_build_node_with::<Self, _>(|graph| {
                    let fields = &[
                        $(
                            UnnamedFieldNode::new::<$ident>(Default::default(), Default::default(), graph),
                        )*
                    ];
                    TupleNode::new::<Self>(fields, Default::default(), Default::default())
                })
            }


        }

        #[allow(non_snake_case, unused_mut, unused_variables)]
        impl<$($ident,)*> Reflect for ($($ident,)*)
        where
            $($ident: Reflect + DescribeType + Clone,)*
        {
            trivial_reflect_methods!();

            #[allow(unused_assignments)]
            fn patch(&mut self, value: &dyn Reflect) {
                if let Some(tuple) = value.reflect_ref().as_tuple() {
                    let ($($ident,)*) = self;
                    let mut i = 0;
                    $(
                        if let Some(field) = tuple.field_at(i) {
                            $ident.patch(field);
                        }
                        i += 1;
                    )*
                }
            }

            fn to_value(&self) -> Value {
                let ($($ident,)*) = self;
                let mut value = TupleValue::new();
                $(
                    value = value.with_field($ident.to_value());
                )*
                value.into()
            }

            fn clone_reflect(&self) -> Box<dyn Reflect> {
                Box::new(self.clone())
            }

            fn debug(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{}", core::any::type_name::<Self>())
            }

            fn reflect_owned(self: Box<Self>) -> ReflectOwned {
                ReflectOwned::Tuple(self)
            }

            fn reflect_ref(&self) -> ReflectRef<'_> {
                ReflectRef::Tuple(self)
            }

            fn reflect_mut(&mut self) -> ReflectMut<'_> {
                ReflectMut::Tuple(self)
            }
        }

        #[allow(non_snake_case, unused_mut, unused_assignments, unused_variables)]
        impl<$($ident,)*> Tuple for ($($ident,)*)
        where
            $($ident: Reflect + DescribeType + Clone,)*
        {
            fn field_at(&self, index: usize) -> Option<&dyn Reflect> {
                let mut i = 0;
                let ($($ident,)*) = self;
                $(
                    if index == i {
                        return Some($ident);
                    }
                    i += 1;
                )*
                None
            }

            fn field_at_mut(&mut self, index: usize) -> Option<&mut dyn Reflect> {
                let mut i = 0;
                let ($($ident,)*) = self;
                $(
                    if index == i {
                        return Some($ident);
                    }
                    i += 1;
                )*
                None
            }

            fn fields(&self) -> Iter<'_> {
                Iter::new(self)
            }

            fn fields_mut(&mut self) -> ValueIterMut<'_> {
                let ($($ident,)*) = self;
                Box::new([$($ident.as_reflect_mut(),)*].into_iter())
            }

            fn fields_len(&self) -> usize {
                let mut n = 0;
                $(
                    let _ = stringify!($ident);
                    n += 1;
                )*
                n
            }
        }

        #[allow(non_snake_case, unused_mut, unused_assignments, unused_variables)]
        impl<$($ident,)*> FromReflect for ($($ident,)*)
        where
            $($ident: FromReflect + DescribeType + Clone,)*
        {
            fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
                let tuple = reflect.as_tuple()?;
                let mut fields = tuple.fields();
                Some((
                    $($ident::from_reflect(fields.next()?)?,)*
                ))
            }
        }
    };
}

impl_tuple!();
impl_tuple!(T1);
impl_tuple!(T1, T2);
impl_tuple!(T1, T2, T3);
impl_tuple!(T1, T2, T3, T4);
impl_tuple!(T1, T2, T3, T4, T5);
impl_tuple!(T1, T2, T3, T4, T5, T6);
impl_tuple!(T1, T2, T3, T4, T5, T6, T7);
impl_tuple!(T1, T2, T3, T4, T5, T6, T7, T8);
impl_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
impl_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
impl_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
impl_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14);
impl_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15);
impl_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16);

impl<V> FromIterator<V> for TupleValue
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
    tuple: &'a dyn Tuple,
    index: usize,
}

impl<'a> Iter<'a> {
    pub fn new(tuple: &'a dyn Tuple) -> Self {
        Self { tuple, index: 0 }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a dyn Reflect;

    fn next(&mut self) -> Option<Self::Item> {
        let value = self.tuple.field_at(self.index)?;
        self.index += 1;
        Some(value)
    }
}

impl<'a> ExactSizeIterator for Iter<'a> {
    fn len(&self) -> usize {
        self.tuple.fields_len()
    }
}

impl<'a> FusedIterator for Iter<'a> {}
