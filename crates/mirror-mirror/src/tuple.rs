use std::{any::Any, fmt::Debug};

use serde::{Deserialize, Serialize};
use speedy::{Readable, Writable};

use crate::{
    Enum, FromReflect, List, Reflect, Struct, TupleStruct, Value, ValueIter, ValueIterMut,
};

pub trait Tuple: Reflect {
    fn element(&self, index: usize) -> Option<&dyn Reflect>;
    fn element_mut(&mut self, index: usize) -> Option<&mut dyn Reflect>;

    fn elements(&self) -> ValueIter<'_>;
    fn elements_mut(&mut self) -> ValueIterMut<'_>;
}

#[derive(Default, Readable, Writable, Serialize, Deserialize, Debug, Clone)]
pub struct TupleValue {
    elements: Vec<Value>,
}

impl TupleValue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_element(mut self, value: impl Into<Value>) -> Self {
        self.push_element(value);
        self
    }

    pub fn push_element(&mut self, value: impl Into<Value>) {
        self.elements.push(value.into());
    }
}

impl Tuple for TupleValue {
    fn element(&self, index: usize) -> Option<&dyn Reflect> {
        Some(self.elements.get(index)?.as_reflect())
    }

    fn element_mut(&mut self, index: usize) -> Option<&mut dyn Reflect> {
        Some(self.elements.get_mut(index)?.as_reflect_mut())
    }

    fn elements(&self) -> ValueIter<'_> {
        let iter = self.elements.iter().map(|value| value.as_reflect());
        ValueIter::new(iter)
    }

    fn elements_mut(&mut self) -> ValueIterMut<'_> {
        let iter = self.elements.iter_mut().map(|value| value.as_reflect_mut());
        ValueIterMut::new(iter)
    }
}

impl Reflect for TupleValue {
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

    fn as_tuple(&self) -> Option<&dyn Tuple> {
        Some(self)
    }

    fn as_tuple_mut(&mut self) -> Option<&mut dyn Tuple> {
        Some(self)
    }

    fn as_struct(&self) -> Option<&dyn Struct> {
        None
    }

    fn as_struct_mut(&mut self) -> Option<&mut dyn Struct> {
        None
    }

    fn as_tuple_struct(&self) -> Option<&dyn TupleStruct> {
        None
    }

    fn as_tuple_struct_mut(&mut self) -> Option<&mut dyn TupleStruct> {
        None
    }

    fn as_enum(&self) -> Option<&dyn Enum> {
        None
    }

    fn as_enum_mut(&mut self) -> Option<&mut dyn Enum> {
        None
    }

    fn as_list(&self) -> Option<&dyn List> {
        None
    }

    fn as_list_mut(&mut self) -> Option<&mut dyn List> {
        None
    }

    fn patch(&mut self, value: &dyn Reflect) {
        if let Some(tuple) = value.as_tuple() {
            for (index, value) in self.elements_mut().enumerate() {
                if let Some(new_value) = tuple.element(index) {
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
}

impl FromReflect for TupleValue {
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
        let tuple = reflect.as_tuple()?;
        let this = tuple
            .elements()
            .fold(TupleValue::default(), |builder, value| {
                builder.with_element(value.to_value())
            });
        Some(this)
    }
}

macro_rules! impl_tuple {
    ($($ident:ident),* $(,)?) => {
        #[allow(non_snake_case, unused_mut, unused_variables)]
        impl<$($ident,)*> Reflect for ($($ident,)*)
        where
            $($ident: Reflect + Clone,)*
        {
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

            fn as_tuple(&self) -> Option<&dyn Tuple> {
                Some(self)
            }

            fn as_tuple_mut(&mut self) -> Option<&mut dyn Tuple> {
                Some(self)
            }

            fn as_struct(&self) -> Option<&dyn Struct> {
                None
            }

            fn as_struct_mut(&mut self) -> Option<&mut dyn Struct> {
                None
            }

            fn as_tuple_struct(&self) -> Option<&dyn TupleStruct> {
                None
            }

            fn as_tuple_struct_mut(&mut self) -> Option<&mut dyn TupleStruct> {
                None
            }

            fn as_enum(&self) -> Option<&dyn Enum> {
                None
            }

            fn as_enum_mut(&mut self) -> Option<&mut dyn Enum> {
                None
            }

            fn as_list(&self) -> Option<&dyn List> {
                None
            }

            fn as_list_mut(&mut self) -> Option<&mut dyn List> {
                None
            }

            #[allow(unused_assignments)]
            fn patch(&mut self, value: &dyn Reflect) {
                if let Some(tuple) = value.as_tuple() {
                    let ($($ident,)*) = self;
                    let mut i = 0;
                    $(
                        if let Some(element) = tuple.element(i) {
                            $ident.patch(element);
                        }
                        i += 1;
                    )*
                }
            }

            fn to_value(&self) -> Value {
                let ($($ident,)*) = self;
                let mut value = TupleValue::new();
                $(
                    value = value.with_element($ident.to_value());
                )*
                value.into()
            }

            fn clone_reflect(&self) -> Box<dyn Reflect> {
                Box::new(self.clone())
            }

            fn debug(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", std::any::type_name::<Self>())
            }
        }

        #[allow(non_snake_case, unused_mut, unused_assignments, unused_variables)]
        impl<$($ident,)*> Tuple for ($($ident,)*)
        where
            $($ident: Reflect + Clone,)*
        {
            fn element(&self, index: usize) -> Option<&dyn Reflect> {
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

            fn element_mut(&mut self, index: usize) -> Option<&mut dyn Reflect> {
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

            fn elements(&self) -> ValueIter<'_> {
                let ($($ident,)*) = self;
                ValueIter::new([$($ident.as_reflect(),)*])
            }

            fn elements_mut(&mut self) -> ValueIterMut<'_> {
                let ($($ident,)*) = self;
                ValueIterMut::new([$($ident.as_reflect_mut(),)*])
            }
        }

        #[allow(non_snake_case, unused_mut, unused_assignments, unused_variables)]
        impl<$($ident,)*> FromReflect for ($($ident,)*)
        where
            $($ident: FromReflect + Clone,)*
        {
            fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
                Some((
                    $($ident::from_reflect(reflect)?,)*
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
            out.push_element(value.to_value());
        }
        out
    }
}
