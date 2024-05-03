use glam::Quat;
use kollect::LinearMap;
use std::any::Any;

use crate::{
    struct_::{FieldsIter, FieldsIterMut, StructValue},
    type_info::graph::*,
    DefaultValue, DescribeType, FromReflect, Reflect, ReflectMut, ReflectOwned, ReflectRef, Struct,
    Value,
};

impl Reflect for Quat {
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
            if let Some(x) = struct_.field("x").and_then(<_>::from_reflect) {
                self.x = x;
            }
            if let Some(y) = struct_.field("y").and_then(<_>::from_reflect) {
                self.y = y;
            }
            if let Some(z) = struct_.field("z").and_then(<_>::from_reflect) {
                self.z = z;
            }
            if let Some(w) = struct_.field("w").and_then(<_>::from_reflect) {
                self.w = w;
            }
        }
    }

    fn to_value(&self) -> Value {
        StructValue::with_capacity(4)
            .with_field("x", self.x)
            .with_field("y", self.y)
            .with_field("z", self.z)
            .with_field("w", self.w)
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

impl Struct for Quat {
    fn field(&self, name: &str) -> Option<&dyn Reflect> {
        match name {
            "x" => Some(&self.x),
            "y" => Some(&self.y),
            "z" => Some(&self.z),
            "w" => Some(&self.w),
            _ => None,
        }
    }

    fn field_mut(&mut self, name: &str) -> Option<&mut dyn Reflect> {
        match name {
            "x" => Some(&mut self.x),
            "y" => Some(&mut self.y),
            "z" => Some(&mut self.z),
            "w" => Some(&mut self.w),
            _ => None,
        }
    }

    fn field_at(&self, index: usize) -> Option<&dyn Reflect> {
        match index {
            0 => Some(&self.x),
            1 => Some(&self.y),
            2 => Some(&self.z),
            3 => Some(&self.w),
            _ => None,
        }
    }

    fn field_at_mut(&mut self, index: usize) -> Option<&mut dyn Reflect> {
        match index {
            0 => Some(&mut self.x),
            1 => Some(&mut self.y),
            2 => Some(&mut self.z),
            3 => Some(&mut self.w),
            _ => None,
        }
    }

    fn name_at(&self, index: usize) -> Option<&str> {
        match index {
            0 => Some("x"),
            1 => Some("y"),
            2 => Some("z"),
            3 => Some("w"),
            _ => None,
        }
    }

    fn fields(&self) -> FieldsIter<'_> {
        Box::new(
            [
                ("x", self.x.as_reflect()),
                ("y", self.y.as_reflect()),
                ("z", self.z.as_reflect()),
                ("w", self.w.as_reflect()),
            ]
            .into_iter(),
        )
    }

    fn fields_mut(&mut self) -> FieldsIterMut<'_> {
        let repr = &mut **self;
        Box::new(
            [
                ("x", repr.x.as_reflect_mut()),
                ("y", repr.y.as_reflect_mut()),
                ("z", repr.z.as_reflect_mut()),
                ("w", repr.w.as_reflect_mut()),
            ]
            .into_iter(),
        )
    }

    fn fields_len(&self) -> usize {
        4
    }
}

impl FromReflect for Quat {
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
        if let Some(quat) = reflect.downcast_ref() {
            Some(*quat)
        } else {
            let struct_ = reflect.as_struct()?;
            let x = <_>::from_reflect(struct_.field("x")?)?;
            let y = <_>::from_reflect(struct_.field("y")?)?;
            let z = <_>::from_reflect(struct_.field("z")?)?;
            let w = <_>::from_reflect(struct_.field("w")?)?;
            Some(Self::from_xyzw(x, y, z, w))
        }
    }
}

impl DefaultValue for Quat {
    fn default_value() -> Option<Value> {
        Some(Self::default().to_value())
    }
}

impl DescribeType for Quat {
    fn build(graph: &mut TypeGraph) -> NodeId {
        graph.get_or_build_node_with::<Self, _>(|graph| {
            let fields = &[
                NamedFieldNode::new::<f32>("x", LinearMap::from([]), &[], graph),
                NamedFieldNode::new::<f32>("y", LinearMap::from([]), &[], graph),
                NamedFieldNode::new::<f32>("z", LinearMap::from([]), &[], graph),
                NamedFieldNode::new::<f32>("w", LinearMap::from([]), &[], graph),
            ];
            StructNode::new::<Self>(fields, LinearMap::from([]), &[])
        })
    }
}
