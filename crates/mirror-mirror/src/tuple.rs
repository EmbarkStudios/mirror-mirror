use std::{any::Any, fmt::Debug};

use serde::{Deserialize, Serialize};
use speedy::{Readable, Writable};

use crate::{Enum, FromReflect, Reflect, Struct, Value};

pub trait Tuple: Reflect {
    fn element(&self, index: usize) -> Option<&dyn Reflect>;
    fn element_mut(&mut self, index: usize) -> Option<&mut dyn Reflect>;

    fn elements(&self) -> TupleFieldsIter<'_>;
    fn elements_mut(&mut self) -> TupleFieldsIterMut<'_>;
}

pub struct TupleFieldsIter<'a> {
    iter: Box<dyn Iterator<Item = &'a dyn Reflect> + 'a>,
}

impl<'a> TupleFieldsIter<'a> {
    pub fn new<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = &'a dyn Reflect> + 'a,
    {
        Self {
            iter: Box::new(iter.into_iter()),
        }
    }
}

impl<'a> Iterator for TupleFieldsIter<'a> {
    type Item = &'a dyn Reflect;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

pub struct TupleFieldsIterMut<'a> {
    iter: Box<dyn Iterator<Item = &'a mut dyn Reflect> + 'a>,
}

impl<'a> TupleFieldsIterMut<'a> {
    pub fn new<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = &'a mut dyn Reflect> + 'a,
    {
        Self {
            iter: Box::new(iter.into_iter()),
        }
    }
}

impl<'a> Iterator for TupleFieldsIterMut<'a> {
    type Item = &'a mut dyn Reflect;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
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
        self.elements.push(value.into());
        self
    }
}

impl Tuple for TupleValue {
    fn element(&self, index: usize) -> Option<&dyn Reflect> {
        Some(self.elements.get(index)?.as_reflect())
    }

    fn element_mut(&mut self, index: usize) -> Option<&mut dyn Reflect> {
        Some(self.elements.get_mut(index)?.as_reflect_mut())
    }

    fn elements(&self) -> TupleFieldsIter<'_> {
        let iter = self.elements.iter().map(|value| value.as_reflect());
        TupleFieldsIter::new(iter)
    }

    fn elements_mut(&mut self) -> TupleFieldsIterMut<'_> {
        let iter = self.elements.iter_mut().map(|value| value.as_reflect_mut());
        TupleFieldsIterMut::new(iter)
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

    fn as_enum(&self) -> Option<&dyn Enum> {
        None
    }

    fn as_enum_mut(&mut self) -> Option<&mut dyn Enum> {
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
        #[allow(non_snake_case)]
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

            fn as_enum(&self) -> Option<&dyn Enum> {
                None
            }

            fn as_enum_mut(&mut self) -> Option<&mut dyn Enum> {
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
                write!(f, "{}", std::any::type_name::<T1>())
            }
        }

        #[allow(non_snake_case, unused_assignments)]
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

            fn elements(&self) -> TupleFieldsIter<'_> {
                let ($($ident,)*) = self;
                TupleFieldsIter::new([$($ident.as_reflect(),)*])
            }

            fn elements_mut(&mut self) -> TupleFieldsIterMut<'_> {
                let ($($ident,)*) = self;
                TupleFieldsIterMut::new([$($ident.as_reflect_mut(),)*])
            }
        }

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
