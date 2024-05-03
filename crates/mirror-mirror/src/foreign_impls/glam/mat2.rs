use glam::{Mat2, Vec2};
use kollect::LinearMap;
use std::any::Any;

use crate::{
    struct_::{FieldsIter, FieldsIterMut, StructValue},
    type_info::graph::*,
    DefaultValue, DescribeType, FromReflect, Reflect, ReflectMut, ReflectOwned, ReflectRef, Struct,
    Value,
};

impl Reflect for Mat2 {
    trivial_reflect_methods!();

    fn reflect_owned(self: Box<Self>) -> ReflectOwned {
        ReflectOwned::Struct(self)
    }

    fn reflect_ref(&self) -> ReflectRef<'_> {
        ReflectRef::Struct(self)
    }

    fn reflect_mut(&mut self) -> ReflectMut<'_> {
        ReflectMut::Struct(self)
    }

    fn patch(&mut self, value: &dyn Reflect) {
        if let Some(struct_) = value.as_struct() {
            if let Some(x_axis) = struct_.field("x_axis").and_then(<_>::from_reflect) {
                self.x_axis = x_axis;
            }
            if let Some(y_axis) = struct_.field("y_axis").and_then(<_>::from_reflect) {
                self.y_axis = y_axis;
            }
        }
    }

    fn to_value(&self) -> Value {
        StructValue::with_capacity(2)
            .with_field("x_axis", self.x_axis)
            .with_field("y_axis", self.y_axis)
            .to_value()
    }

    fn clone_reflect(&self) -> Box<dyn Reflect> {
        Box::new(*self)
    }

    fn debug(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if f.alternate() {
            write!(f, "{self:#?}")
        } else {
            write!(f, "{self:?}")
        }
    }
}

impl Struct for Mat2 {
    fn field(&self, name: &str) -> Option<&dyn Reflect> {
        match name {
            "x_axis" => Some(&self.x_axis),
            "y_axis" => Some(&self.y_axis),
            _ => None,
        }
    }

    fn field_mut(&mut self, name: &str) -> Option<&mut dyn Reflect> {
        match name {
            "x_axis" => Some(&mut self.x_axis),
            "y_axis" => Some(&mut self.y_axis),
            _ => None,
        }
    }

    fn field_at(&self, index: usize) -> Option<&dyn Reflect> {
        match index {
            0 => Some(&self.x_axis),
            1 => Some(&self.y_axis),
            _ => None,
        }
    }

    fn field_at_mut(&mut self, index: usize) -> Option<&mut dyn Reflect> {
        match index {
            0 => Some(&mut self.x_axis),
            1 => Some(&mut self.y_axis),
            _ => None,
        }
    }

    fn name_at(&self, index: usize) -> Option<&str> {
        match index {
            0 => Some("x_axis"),
            1 => Some("y_axis"),
            _ => None,
        }
    }

    fn fields(&self) -> FieldsIter<'_> {
        Box::new(
            [
                ("x_axis", self.x_axis.as_reflect()),
                ("y_axis", self.y_axis.as_reflect()),
            ]
            .into_iter(),
        )
    }

    fn fields_mut(&mut self) -> FieldsIterMut<'_> {
        let repr = &mut **self;
        Box::new(
            [
                ("x_axis", repr.x_axis.as_reflect_mut()),
                ("y_axis", repr.y_axis.as_reflect_mut()),
            ]
            .into_iter(),
        )
    }

    fn fields_len(&self) -> usize {
        2
    }
}

impl FromReflect for Mat2 {
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
        if let Some(mat) = reflect.downcast_ref() {
            Some(*mat)
        } else {
            let struct_ = reflect.as_struct()?;
            let x_axis = <_>::from_reflect(struct_.field("x_axis")?)?;
            let y_axis = <_>::from_reflect(struct_.field("y_axis")?)?;
            Some(Self::from_cols(x_axis, y_axis))
        }
    }
}

impl DefaultValue for Mat2 {
    fn default_value() -> Option<Value> {
        Some(Self::default().to_value())
    }
}

impl DescribeType for Mat2 {
    fn build(graph: &mut TypeGraph) -> NodeId {
        graph.get_or_build_node_with::<Self, _>(|graph| {
            let fields = &[
                NamedFieldNode::new::<Vec2>("x_axis", LinearMap::from([]), &[], graph),
                NamedFieldNode::new::<Vec2>("y_axis", LinearMap::from([]), &[], graph),
            ];
            StructNode::new::<Self>(fields, LinearMap::from([]), &[])
        })
    }
}
