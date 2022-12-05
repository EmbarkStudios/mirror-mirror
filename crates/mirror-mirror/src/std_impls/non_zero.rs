use std::{
    any::Any,
    fmt,
    num::{
        NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroU128, NonZeroU16,
        NonZeroU32, NonZeroU64, NonZeroU8, NonZeroUsize,
    },
};

use crate::{
    type_info::graph::{Id, TypeInfoGraph},
    FromReflect, Reflect, ReflectMut, ReflectRef, ScalarRef, TypeInfoRoot, Typed, Value,
};

macro_rules! impl_traits {
    ($($non_zero_ident:ident ($inner:ident)),* $(,)?) => {
        $(
            impl Reflect for $non_zero_ident {
                fn type_info(&self) -> TypeInfoRoot {
                    impl Typed for $non_zero_ident {
                        fn build(graph: &mut TypeInfoGraph) -> Id {
                            <$inner as Typed>::build(graph)
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

                fn reflect_ref(&self) -> ReflectRef<'_> {
                    ReflectRef::Scalar(ScalarRef::$inner(self.get()))
                }

                fn reflect_mut(&mut self) -> ReflectMut<'_> {
                    ReflectMut::Opaque(self)
                }

                fn patch(&mut self, value: &dyn Reflect) {
                    if let Some(n) = Self::from_reflect(value) {
                        *self = n;
                    }
                }

                fn to_value(&self) -> Value {
                    self.get().to_value()
                }

                fn clone_reflect(&self) -> Box<dyn Reflect> {
                    Box::new(*self)
                }

                fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    if f.alternate() {
                        write!(f, "{:#?}", self)
                    } else {
                        write!(f, "{:?}", self)
                    }
                }
            }

            impl FromReflect for $non_zero_ident {
                fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
                    if let Some(n) = reflect.downcast_ref::<Self>() {
                        Some(*n)
                    } else {
                        $inner::from_reflect(reflect).and_then(Self::new)
                    }
                }
            }

            impl From<$non_zero_ident> for Value {
                fn from(n: $non_zero_ident) -> Self {
                    n.to_value()
                }
            }
        )*
    };
}

impl_traits! {
    NonZeroUsize(usize),
    NonZeroU8(u8),
    NonZeroU16(u16),
    NonZeroU32(u32),
    NonZeroU64(u64),
    NonZeroU128(u128),
    NonZeroI8(i8),
    NonZeroI16(i16),
    NonZeroI32(i32),
    NonZeroI64(i64),
    NonZeroI128(i128),
}
