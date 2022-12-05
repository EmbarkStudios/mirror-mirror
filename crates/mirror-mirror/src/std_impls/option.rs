use crate::enum_::EnumValue;
use crate::enum_::VariantFieldIter;
use crate::enum_::VariantFieldIterMut;
use crate::enum_::VariantFieldMut;
use crate::enum_::VariantKind;
use crate::type_info::graph::EnumInfoNode;
use crate::type_info::graph::Id;
use crate::type_info::graph::TupleVariantInfoNode;
use crate::type_info::graph::TypeInfoGraph;
use crate::type_info::graph::UnitVariantInfoNode;
use crate::type_info::graph::UnnamedFieldNode;
use crate::type_info::graph::VariantNode;
use crate::Enum;
use crate::FromReflect;
use crate::Reflect;
use crate::ReflectMut;
use crate::ReflectRef;
use crate::TypeInfoRoot;
use crate::Typed;
use crate::Value;
use std::any::Any;
use std::fmt;

impl<T> Reflect for Option<T>
where
    T: FromReflect + Typed,
{
    fn type_info(&self) -> TypeInfoRoot {
        impl<T> Typed for Option<T>
        where
            T: Typed,
        {
            fn build(graph: &mut TypeInfoGraph) -> Id {
                graph.get_or_build_with::<Self, _>(|graph| {
                    EnumInfoNode::new::<Self>(
                        &[
                            VariantNode::Tuple(TupleVariantInfoNode::new(
                                "Some",
                                &[UnnamedFieldNode::new::<T>(Default::default(), graph)],
                                Default::default(),
                            )),
                            VariantNode::Unit(UnitVariantInfoNode::new("None", Default::default())),
                        ],
                        Default::default(),
                    )
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
                for (index, value) in self.fields_mut().enumerate() {
                    match value {
                        VariantFieldMut::Struct(_, _) => {}
                        VariantFieldMut::Tuple(value) => {
                            if let Some(new_value) = enum_.field_at(index) {
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
                .with_tuple_field(value.to_value())
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

    fn reflect_ref(&self) -> ReflectRef<'_> {
        ReflectRef::Enum(self)
    }

    fn reflect_mut(&mut self) -> ReflectMut<'_> {
        ReflectMut::Enum(self)
    }
}

impl<T> Enum for Option<T>
where
    T: FromReflect + Typed,
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

    fn field_at(&self, index: usize) -> Option<&dyn Reflect> {
        match self {
            Some(value) if index == 0 => Some(value),
            _ => None,
        }
    }

    fn field_at_mut(&mut self, index: usize) -> Option<&mut dyn Reflect> {
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
    T: FromReflect + Typed,
{
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
        let enum_ = reflect.reflect_ref().as_enum()?;
        match enum_.variant_name() {
            "Some" => {
                let value = enum_.field_at(0)?;
                Some(Some(T::from_reflect(value)?))
            }
            "None" => Some(None),
            _ => None,
        }
    }
}

impl<T> From<Option<T>> for Value
where
    T: Reflect,
{
    fn from(option: Option<T>) -> Self {
        option.map(|inner| inner.to_value()).to_value()
    }
}
