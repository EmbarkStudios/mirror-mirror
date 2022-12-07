use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::string::String;
use core::any::Any;
use core::fmt;

use serde::Deserialize;
use serde::Serialize;

use crate::iter::PairIterMut;
use crate::iter::ValueIterMut;
use crate::struct_::StructValue;
use crate::tuple::TupleValue;
use crate::type_info::graph::Id;
use crate::type_info::graph::OpaqueInfoNode;
use crate::type_info::graph::TypeInfoGraph;
use crate::FromReflect;
use crate::Reflect;
use crate::ReflectMut;
use crate::ReflectRef;
use crate::Struct;
use crate::Tuple;
use crate::TypeInfoRoot;
use crate::Typed;
use crate::Value;

pub trait Enum: Reflect {
    fn variant_name(&self) -> &str;

    fn variant_kind(&self) -> VariantKind;

    fn field(&self, name: &str) -> Option<&dyn Reflect>;

    fn field_mut(&mut self, name: &str) -> Option<&mut dyn Reflect>;

    fn field_at(&self, index: usize) -> Option<&dyn Reflect>;

    fn name_at(&self, index: usize) -> Option<&str>;

    fn field_at_mut(&mut self, index: usize) -> Option<&mut dyn Reflect>;

    fn fields(&self) -> VariantFieldIter<'_>;

    fn fields_mut(&mut self) -> VariantFieldIterMut<'_>;

    fn variants_len(&self) -> usize;

    fn fields_len(&self) -> usize;
}

impl fmt::Debug for dyn Enum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_reflect().debug(f)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VariantKind {
    Struct,
    Tuple,
    Unit,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "speedy", derive(speedy::Readable, speedy::Writable))]
pub struct EnumValue {
    name: String,
    kind: EnumValueKind,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "speedy", derive(speedy::Readable, speedy::Writable))]
enum EnumValueKind {
    Struct(StructValue),
    Tuple(TupleValue),
    Unit,
}

impl EnumValue {
    pub fn new_struct_variant(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: EnumValueKind::Struct(Default::default()),
        }
    }

    pub fn new_tuple_variant(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: EnumValueKind::Tuple(Default::default()),
        }
    }

    pub fn new_unit_variant(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: EnumValueKind::Unit,
        }
    }

    #[track_caller]
    pub fn with_struct_field(mut self, name: impl Into<String>, value: impl Into<Value>) -> Self {
        self.set_struct_field(name, value);
        self
    }

    #[track_caller]
    pub fn with_tuple_field(mut self, value: impl Into<Value>) -> Self {
        self.push_tuple_field(value);
        self
    }

    #[track_caller]
    pub fn set_struct_field(&mut self, name: impl Into<String>, value: impl Into<Value>) {
        match &mut self.kind {
            EnumValueKind::Struct(struct_) => {
                struct_.set_field(name, value);
            }
            EnumValueKind::Tuple(_) => panic!("Cannot set fields on tuple variants"),
            EnumValueKind::Unit => panic!("Cannot set fields on unit variants"),
        }
    }

    #[track_caller]
    pub fn push_tuple_field(&mut self, value: impl Into<Value>) {
        match &mut self.kind {
            EnumValueKind::Struct(_) => {
                panic!("Cannot push fields on struct variants")
            }
            EnumValueKind::Tuple(tuple) => {
                tuple.push_field(value);
            }
            EnumValueKind::Unit => panic!("Cannot set fields on unit variants"),
        }
    }
}

impl Reflect for EnumValue {
    fn type_info(&self) -> TypeInfoRoot {
        impl Typed for EnumValue {
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
        if let Some(enum_) = value.reflect_ref().as_enum() {
            if self.variant_name() == enum_.variant_name() {
                for (idx, field) in self.fields_mut().enumerate() {
                    match field {
                        VariantFieldMut::Struct(name, value) => {
                            if let Some(new_value) = enum_.field(name) {
                                value.patch(new_value);
                            }
                        }
                        VariantFieldMut::Tuple(value) => {
                            if let Some(new_value) = enum_.field_at(idx) {
                                value.patch(new_value);
                            }
                        }
                    }
                }
            } else if let Some(new) = Self::from_reflect(value) {
                *self = new;
            }
        }
    }

    fn to_value(&self) -> Value {
        self.clone().into()
    }

    fn clone_reflect(&self) -> Box<dyn Reflect> {
        Box::new(self.clone())
    }

    fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "{:#?}", self)
        } else {
            write!(f, "{:?}", self)
        }
    }

    fn reflect_ref(&self) -> ReflectRef<'_> {
        ReflectRef::Enum(self)
    }

    fn reflect_mut(&mut self) -> ReflectMut<'_> {
        ReflectMut::Enum(self)
    }
}

impl Enum for EnumValue {
    fn variant_name(&self) -> &str {
        &self.name
    }

    fn variant_kind(&self) -> VariantKind {
        match &self.kind {
            EnumValueKind::Struct(_) => VariantKind::Struct,
            EnumValueKind::Tuple(_) => VariantKind::Tuple,
            EnumValueKind::Unit => VariantKind::Unit,
        }
    }

    fn field(&self, name: &str) -> Option<&dyn Reflect> {
        match &self.kind {
            EnumValueKind::Struct(struct_) => struct_.field(name),
            EnumValueKind::Tuple(_) => None,
            EnumValueKind::Unit => None,
        }
    }

    fn field_mut(&mut self, name: &str) -> Option<&mut dyn Reflect> {
        match &mut self.kind {
            EnumValueKind::Struct(struct_) => struct_.field_mut(name),
            EnumValueKind::Tuple(_) => None,
            EnumValueKind::Unit => None,
        }
    }

    fn field_at(&self, index: usize) -> Option<&dyn Reflect> {
        match &self.kind {
            EnumValueKind::Struct(struct_) => struct_.field_at(index),
            EnumValueKind::Tuple(tuple) => tuple.field(index),
            EnumValueKind::Unit => None,
        }
    }

    fn field_at_mut(&mut self, index: usize) -> Option<&mut dyn Reflect> {
        match &mut self.kind {
            EnumValueKind::Struct(struct_) => struct_.field_at_mut(index),
            EnumValueKind::Tuple(tuple) => tuple.field_mut(index),
            EnumValueKind::Unit => None,
        }
    }

    fn fields(&self) -> VariantFieldIter<'_> {
        VariantFieldIter::new(self)
    }

    fn fields_mut(&mut self) -> VariantFieldIterMut<'_> {
        match &mut self.kind {
            EnumValueKind::Struct(inner) => {
                VariantFieldIterMut(VariantFieldIterInnerMut::Struct(inner.fields_mut()))
            }
            EnumValueKind::Tuple(inner) => {
                VariantFieldIterMut(VariantFieldIterInnerMut::Tuple(inner.fields_mut()))
            }
            EnumValueKind::Unit => VariantFieldIterMut::empty(),
        }
    }

    fn variants_len(&self) -> usize {
        1
    }

    fn fields_len(&self) -> usize {
        match &self.kind {
            EnumValueKind::Struct(inner) => inner.fields_len(),
            EnumValueKind::Tuple(inner) => inner.fields_len(),
            EnumValueKind::Unit => 0,
        }
    }

    fn name_at(&self, index: usize) -> Option<&str> {
        match &self.kind {
            EnumValueKind::Struct(inner) => inner.name_at(index),
            EnumValueKind::Tuple(_) => None,
            EnumValueKind::Unit => None,
        }
    }
}

impl FromReflect for EnumValue {
    #[track_caller]
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
        let enum_ = reflect.reflect_ref().as_enum()?;

        let kind = match enum_.variant_kind() {
            VariantKind::Struct => {
                let struct_ = enum_
                    .fields()
                    .fold(StructValue::default(), |builder, field| match field {
                        VariantField::Struct(name, value) => {
                            builder.with_field(name, value.to_value())
                        }
                        VariantField::Tuple(_) => {
                            panic!("iterator over fields in struct variant yielded a tuple field")
                        }
                    });
                EnumValueKind::Struct(struct_)
            }
            VariantKind::Tuple => {
                let tuple =
                    enum_
                        .fields()
                        .fold(TupleValue::default(), |builder, field| match field {
                            VariantField::Struct(_, _) => {
                                panic!(
                                    "iterator over fields in tuple variant yielded a struct field"
                                )
                            }
                            VariantField::Tuple(value) => builder.with_field(value.to_value()),
                        });
                EnumValueKind::Tuple(tuple)
            }
            VariantKind::Unit => EnumValueKind::Unit,
        };

        Some(EnumValue {
            name: enum_.variant_name().to_owned(),
            kind,
        })
    }
}

#[derive(Debug)]
pub struct VariantFieldIter<'a> {
    enum_: &'a dyn Enum,
    index: usize,
}

impl<'a> VariantFieldIter<'a> {
    pub fn new(enum_: &'a dyn Enum) -> Self {
        Self { enum_, index: 0 }
    }
}

impl<'a> Iterator for VariantFieldIter<'a> {
    type Item = VariantField<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let item = match self.enum_.variant_kind() {
            VariantKind::Struct => {
                let name = self.enum_.name_at(self.index)?;
                let value = self.enum_.field_at(self.index)?;
                VariantField::Struct(name, value)
            }
            VariantKind::Tuple => {
                let value = self.enum_.field_at(self.index)?;
                VariantField::Tuple(value)
            }
            VariantKind::Unit => return None,
        };
        self.index += 1;
        Some(item)
    }
}

#[derive(Debug)]
pub enum VariantField<'a> {
    Struct(&'a str, &'a dyn Reflect),
    Tuple(&'a dyn Reflect),
}

#[derive(Debug)]
pub struct VariantFieldIterMut<'a>(VariantFieldIterInnerMut<'a>);

impl<'a> VariantFieldIterMut<'a> {
    pub fn new_struct_variant<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (&'a str, &'a mut dyn Reflect)> + 'a,
    {
        Self(VariantFieldIterInnerMut::Struct(Box::new(iter.into_iter())))
    }

    pub fn new_tuple_variant<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = &'a mut dyn Reflect> + 'a,
    {
        Self(VariantFieldIterInnerMut::Tuple(Box::new(iter.into_iter())))
    }

    pub fn empty() -> Self {
        Self(VariantFieldIterInnerMut::Empty)
    }
}

enum VariantFieldIterInnerMut<'a> {
    Struct(PairIterMut<'a>),
    Tuple(ValueIterMut<'a>),
    Empty,
}

impl core::fmt::Debug for VariantFieldIterInnerMut<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Struct(_) => f.debug_tuple("Struct").finish(),
            Self::Tuple(_) => f.debug_tuple("Tuple").finish(),
            Self::Empty => write!(f, "Empty"),
        }
    }
}

#[derive(Debug)]
pub enum VariantFieldMut<'a> {
    Struct(&'a str, &'a mut dyn Reflect),
    Tuple(&'a mut dyn Reflect),
}

impl<'a> Iterator for VariantFieldIterMut<'a> {
    type Item = VariantFieldMut<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.0 {
            VariantFieldIterInnerMut::Struct(iter) => iter
                .next()
                .map(|(name, value)| VariantFieldMut::Struct(name, value)),
            VariantFieldIterInnerMut::Tuple(iter) => iter.next().map(VariantFieldMut::Tuple),
            VariantFieldIterInnerMut::Empty => None,
        }
    }
}
