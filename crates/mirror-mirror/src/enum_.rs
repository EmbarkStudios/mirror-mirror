use crate::{
    FromReflect, PairIter, PairIterMut, Reflect, Struct, StructValue, Tuple, TupleStruct,
    TupleValue, Value, ValueIter, ValueIterMut,
};
use serde::{Deserialize, Serialize};
use speedy::{Readable, Writable};
use std::{any::Any, fmt};

pub trait Enum: Reflect {
    fn variant_name(&self) -> &str;
    fn variant_kind(&self) -> VariantKind;

    fn field(&self, name: &str) -> Option<&dyn Reflect>;
    fn field_mut(&mut self, name: &str) -> Option<&mut dyn Reflect>;

    fn element(&self, index: usize) -> Option<&dyn Reflect>;
    fn element_mut(&mut self, index: usize) -> Option<&mut dyn Reflect>;

    fn fields(&self) -> VariantFieldIter<'_>;
    fn fields_mut(&mut self) -> VariantFieldIterMut<'_>;
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum VariantKind {
    Struct,
    Tuple,
    Unit,
}

#[derive(Clone, Debug, Serialize, Deserialize, Writable, Readable)]
pub struct EnumValue {
    name: String,
    kind: EnumValueKind,
}

#[derive(Clone, Debug, Serialize, Deserialize, Writable, Readable)]
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
    pub fn with_field(mut self, name: impl Into<String>, value: impl Into<Value>) -> Self {
        self.set_field(name, value);
        self
    }

    #[track_caller]
    pub fn with_element(mut self, value: impl Into<Value>) -> Self {
        self.push_element(value);
        self
    }

    #[track_caller]
    pub fn set_field(&mut self, name: impl Into<String>, value: impl Into<Value>) {
        match &mut self.kind {
            EnumValueKind::Struct(struct_) => {
                struct_.set_field(name, value);
            }
            EnumValueKind::Tuple(_) => panic!("Cannot set fields on tuple variants"),
            EnumValueKind::Unit => panic!("Cannot set fields on unit variants"),
        }
    }

    #[track_caller]
    pub fn push_element(&mut self, value: impl Into<Value>) {
        match &mut self.kind {
            EnumValueKind::Struct(_) => {
                panic!("Cannot push elements on struct variants")
            }
            EnumValueKind::Tuple(tuple) => {
                tuple.push_element(value);
            }
            EnumValueKind::Unit => panic!("Cannot set fields on unit variants"),
        }
    }
}

impl Reflect for EnumValue {
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
        if let Some(enum_) = value.as_enum() {
            if self.variant_name() == enum_.variant_name() {
                for (idx, field) in self.fields_mut().enumerate() {
                    match field {
                        VariantFieldMut::Struct(name, value) => {
                            if let Some(new_value) = enum_.field(name) {
                                value.patch(new_value);
                            }
                        }
                        VariantFieldMut::Tuple(value) => {
                            if let Some(new_value) = enum_.element(idx) {
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

    fn as_tuple(&self) -> Option<&dyn Tuple> {
        None
    }

    fn as_tuple_mut(&mut self) -> Option<&mut dyn Tuple> {
        None
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
        Some(self)
    }

    fn as_enum_mut(&mut self) -> Option<&mut dyn Enum> {
        Some(self)
    }

    fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "{:#?}", self)
        } else {
            write!(f, "{:?}", self)
        }
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

    fn element(&self, index: usize) -> Option<&dyn Reflect> {
        match &self.kind {
            EnumValueKind::Struct(_) => None,
            EnumValueKind::Tuple(tuple) => tuple.element(index),
            EnumValueKind::Unit => None,
        }
    }

    fn element_mut(&mut self, index: usize) -> Option<&mut dyn Reflect> {
        match &mut self.kind {
            EnumValueKind::Struct(_) => None,
            EnumValueKind::Tuple(tuple) => tuple.element_mut(index),
            EnumValueKind::Unit => None,
        }
    }

    fn fields(&self) -> VariantFieldIter<'_> {
        match &self.kind {
            EnumValueKind::Struct(inner) => {
                VariantFieldIter(VariantFieldIterInner::Struct(inner.fields()))
            }
            EnumValueKind::Tuple(inner) => {
                VariantFieldIter(VariantFieldIterInner::Tuple(inner.elements()))
            }
            EnumValueKind::Unit => VariantFieldIter::empty(),
        }
    }

    fn fields_mut(&mut self) -> VariantFieldIterMut<'_> {
        match &mut self.kind {
            EnumValueKind::Struct(inner) => {
                VariantFieldIterMut(VariantFieldIterInnerMut::Struct(inner.fields_mut()))
            }
            EnumValueKind::Tuple(inner) => {
                VariantFieldIterMut(VariantFieldIterInnerMut::Tuple(inner.elements_mut()))
            }
            EnumValueKind::Unit => VariantFieldIterMut::empty(),
        }
    }
}

impl FromReflect for EnumValue {
    #[track_caller]
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
        let enum_ = reflect.as_enum()?;

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
                            VariantField::Tuple(value) => builder.with_element(value.to_value()),
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

pub struct VariantFieldIter<'a>(VariantFieldIterInner<'a>);

impl<'a> VariantFieldIter<'a> {
    pub fn new_struct_variant<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (&'a str, &'a dyn Reflect)> + 'a,
    {
        Self(VariantFieldIterInner::Struct(PairIter::new(iter)))
    }

    pub fn new_tuple_variant<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = &'a dyn Reflect> + 'a,
    {
        Self(VariantFieldIterInner::Tuple(ValueIter::new(iter)))
    }

    pub fn empty() -> Self {
        Self(VariantFieldIterInner::Empty)
    }
}

enum VariantFieldIterInner<'a> {
    Struct(PairIter<'a>),
    Tuple(ValueIter<'a>),
    Empty,
}

pub enum VariantField<'a> {
    Struct(&'a str, &'a dyn Reflect),
    Tuple(&'a dyn Reflect),
}

impl<'a> Iterator for VariantFieldIter<'a> {
    type Item = VariantField<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.0 {
            VariantFieldIterInner::Struct(iter) => iter
                .next()
                .map(|(name, value)| VariantField::Struct(name, value)),
            VariantFieldIterInner::Tuple(iter) => iter.next().map(VariantField::Tuple),
            VariantFieldIterInner::Empty => None,
        }
    }
}

pub struct VariantFieldIterMut<'a>(VariantFieldIterInnerMut<'a>);

impl<'a> VariantFieldIterMut<'a> {
    pub fn new_struct_variant<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (&'a str, &'a mut dyn Reflect)> + 'a,
    {
        Self(VariantFieldIterInnerMut::Struct(PairIterMut::new(iter)))
    }

    pub fn new_tuple_variant<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = &'a mut dyn Reflect> + 'a,
    {
        Self(VariantFieldIterInnerMut::Tuple(ValueIterMut::new(iter)))
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

impl<T> Reflect for Option<T>
where
    T: FromReflect,
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
        None
    }

    fn as_tuple_mut(&mut self) -> Option<&mut dyn Tuple> {
        None
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
        Some(self)
    }

    fn as_enum_mut(&mut self) -> Option<&mut dyn Enum> {
        Some(self)
    }

    fn patch(&mut self, value: &dyn Reflect) {
        if let Some(enum_) = value.as_enum() {
            if self.variant_name() == enum_.variant_name() {
                for (index, value) in self.fields_mut().enumerate() {
                    match value {
                        VariantFieldMut::Struct(_, _) => {}
                        VariantFieldMut::Tuple(value) => {
                            if let Some(new_value) = enum_.element(index) {
                                value.patch(new_value);
                            }
                        }
                    }
                }
            } else if let Some(new) = Self::from_reflect(enum_.as_reflect()) {
                *self = new;
            }
        }
    }

    fn to_value(&self) -> Value {
        match self {
            Some(value) => EnumValue::new_tuple_variant("Some")
                .with_element(value.to_value())
                .into(),
            None => EnumValue::new_unit_variant("None").into(),
        }
    }

    fn clone_reflect(&self) -> Box<dyn Reflect> {
        let value = self.to_value();
        Box::new(Self::from_reflect(&value).unwrap())
    }

    fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Some(value) => {
                write!(f, "Some(")?;
                value.debug(f)?;
                write!(f, ")")
            }
            None => write!(f, "None"),
        }
    }
}

impl<T> Enum for Option<T>
where
    T: FromReflect,
{
    fn variant_name(&self) -> &str {
        match self {
            Some(_) => "Some",
            None => "None",
        }
    }

    fn variant_kind(&self) -> VariantKind {
        match self {
            Some(_) => VariantKind::Tuple,
            None => VariantKind::Unit,
        }
    }

    fn field(&self, _name: &str) -> Option<&dyn Reflect> {
        None
    }

    fn field_mut(&mut self, _name: &str) -> Option<&mut dyn Reflect> {
        None
    }

    fn element(&self, index: usize) -> Option<&dyn Reflect> {
        match self {
            Some(value) if index == 0 => Some(value),
            _ => None,
        }
    }

    fn element_mut(&mut self, index: usize) -> Option<&mut dyn Reflect> {
        match self {
            Some(value) if index == 0 => Some(value),
            _ => None,
        }
    }

    fn fields(&self) -> VariantFieldIter<'_> {
        match self {
            Some(value) => VariantFieldIter::new_tuple_variant([value.as_reflect()]),
            None => VariantFieldIter::empty(),
        }
    }

    fn fields_mut(&mut self) -> VariantFieldIterMut<'_> {
        match self {
            Some(value) => VariantFieldIterMut::new_tuple_variant([value.as_reflect_mut()]),
            None => VariantFieldIterMut::empty(),
        }
    }
}

impl<T> FromReflect for Option<T>
where
    T: FromReflect,
{
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
        let enum_ = reflect.as_enum()?;
        match enum_.variant_name() {
            "Some" => {
                let value = enum_.element(0)?;
                Some(Some(T::from_reflect(value)?))
            }
            "None" => Some(None),
            _ => None,
        }
    }
}
