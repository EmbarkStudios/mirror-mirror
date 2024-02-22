use core::convert::Infallible;
use core::ops::Range;
use core::ops::RangeFrom;
use core::ops::RangeFull;
use core::ops::RangeTo;
use core::ops::RangeToInclusive;

use crate::__private::*;
use mirror_mirror_macros::__private_derive_reflect_foreign;

mod array;
mod boxed;
mod btree_map;
mod vec;
mod via_scalar;

#[cfg(feature = "glam")]
mod glam;
#[cfg(feature = "macaw")]
mod macaw;

__private_derive_reflect_foreign! {
    #[reflect(opt_out(Clone, Debug), crate_name(crate))]
    enum Option<T>
    where
        T: FromReflect + DescribeType,
    {
        None,
        Some(T),
    }
}

__private_derive_reflect_foreign! {
    #[reflect(opt_out(Clone, Debug, Default), crate_name(crate))]
    enum Result<T, E>
    where
        T: FromReflect + DescribeType,
        E: FromReflect + DescribeType,
    {
        Ok(T),
        Err(E),
    }
}

__private_derive_reflect_foreign! {
    #[reflect(opt_out(Clone, Debug, Default), crate_name(crate))]
    struct Range<Idx>
    where
        Idx: FromReflect + DescribeType
    {
        start: Idx,
        end: Idx,
    }
}

__private_derive_reflect_foreign! {
    #[reflect(opt_out(Clone, Debug, Default), crate_name(crate))]
    struct RangeFrom<Idx>
    where
        Idx: FromReflect + DescribeType,
    {
        start: Idx,
    }
}

__private_derive_reflect_foreign! {
    #[reflect(crate_name(crate))]
    struct RangeFull;
}

__private_derive_reflect_foreign! {
    #[reflect(opt_out(Clone, Debug, Default), crate_name(crate))]
    struct RangeToInclusive<Idx>
    where
        Idx: FromReflect + DescribeType,
    {
        end: Idx,
    }
}

__private_derive_reflect_foreign! {
    #[reflect(opt_out(Clone, Debug, Default), crate_name(crate))]
    struct RangeTo<Idx>
    where
        Idx: FromReflect + DescribeType,
    {
        end: Idx,
    }
}

impl DescribeType for Infallible {
    fn build(graph: &mut TypeGraph) -> NodeId {
        let variants = &[];
        graph.get_or_build_node_with::<Self, _>(|_graph| {
            EnumNode::new::<Self>(variants, BTreeMap::from([]), &[])
        })
    }
}

impl DefaultValue for Infallible {
    fn default_value() -> Option<Value> {
        None
    }
}

impl Reflect for Infallible {
    fn as_any(&self) -> &dyn Any {
        match *self {}
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        match *self {}
    }

    fn as_reflect(&self) -> &dyn Reflect {
        match *self {}
    }

    fn as_reflect_mut(&mut self) -> &mut dyn Reflect {
        match *self {}
    }

    fn type_descriptor(&self) -> Cow<'static, TypeDescriptor> {
        <Self as DescribeType>::type_descriptor()
    }

    fn patch(&mut self, _value: &dyn Reflect) {
        match *self {}
    }

    fn to_value(&self) -> Value {
        match *self {}
    }

    fn clone_reflect(&self) -> Box<dyn Reflect> {
        match *self {}
    }

    fn debug(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {}
    }

    fn reflect_owned(self: Box<Self>) -> ReflectOwned {
        match *self {}
    }

    fn reflect_ref(&self) -> ReflectRef<'_> {
        match *self {}
    }

    fn reflect_mut(&mut self) -> ReflectMut<'_> {
        match *self {}
    }
}

impl FromReflect for Infallible {
    fn from_reflect(_reflect: &dyn Reflect) -> Option<Self> {
        None
    }
}

impl Enum for Infallible {
    fn variant_name(&self) -> &str {
        match *self {}
    }

    fn variant_kind(&self) -> VariantKind {
        match *self {}
    }

    fn field(&self, _name: &str) -> Option<&dyn Reflect> {
        match *self {}
    }

    fn field_mut(&mut self, _name: &str) -> Option<&mut dyn Reflect> {
        match *self {}
    }

    fn field_at(&self, _index: usize) -> Option<&dyn Reflect> {
        match *self {}
    }

    fn field_at_mut(&mut self, _index: usize) -> Option<&mut dyn Reflect> {
        match *self {}
    }

    fn fields(&self) -> crate::enum_::VariantFieldIter<'_> {
        match *self {}
    }

    fn fields_mut(&mut self) -> VariantFieldIterMut<'_> {
        match *self {}
    }

    fn variants_len(&self) -> usize {
        match *self {}
    }

    fn fields_len(&self) -> usize {
        match *self {}
    }

    fn name_at(&self, _index: usize) -> Option<&str> {
        match *self {}
    }
}

impl From<Infallible> for Value {
    fn from(value: Infallible) -> Value {
        match value {}
    }
}
