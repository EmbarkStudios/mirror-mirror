use std::num::{
    NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroU128, NonZeroU16,
    NonZeroU32, NonZeroU64, NonZeroU8, NonZeroUsize,
};

macro_rules! impl_reflect_via_scalar {
    ($ty:ty, $via_ty:ty, $get_fn:expr $(,)?) => {
        const _: () = {
            use $crate::__private::*;

            impl Reflect for $ty {
                fn type_info(&self) -> TypeInfoRoot {
                    impl Typed for $ty {
                        fn build(graph: &mut TypeInfoGraph) -> Id {
                            <$via_ty as Typed>::build(graph)
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
                    ReflectRef::Scalar(ScalarRef::from($get_fn(self)))
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
                    $get_fn(self).to_value()
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

            impl FromReflect for $ty {
                fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
                    if let Some(n) = reflect.downcast_ref::<Self>() {
                        Some(*n)
                    } else {
                        <$via_ty>::from_reflect(reflect).and_then(Self::new)
                    }
                }
            }

            impl From<$ty> for Value {
                fn from(n: $ty) -> Self {
                    n.to_value()
                }
            }
        };
    };
}

impl_reflect_via_scalar! { NonZeroUsize, usize, |n: &NonZeroUsize| n.get() }
impl_reflect_via_scalar! { NonZeroU8,    u8,    |n: &NonZeroU8| n.get()    }
impl_reflect_via_scalar! { NonZeroU16,   u16,   |n: &NonZeroU16| n.get()   }
impl_reflect_via_scalar! { NonZeroU32,   u32,   |n: &NonZeroU32| n.get()   }
impl_reflect_via_scalar! { NonZeroU64,   u64,   |n: &NonZeroU64| n.get()   }
impl_reflect_via_scalar! { NonZeroU128,  u128,  |n: &NonZeroU128| n.get()  }
impl_reflect_via_scalar! { NonZeroI8,    i8,    |n: &NonZeroI8| n.get()    }
impl_reflect_via_scalar! { NonZeroI16,   i16,   |n: &NonZeroI16| n.get()   }
impl_reflect_via_scalar! { NonZeroI32,   i32,   |n: &NonZeroI32| n.get()   }
impl_reflect_via_scalar! { NonZeroI64,   i64,   |n: &NonZeroI64| n.get()   }
impl_reflect_via_scalar! { NonZeroI128,  i128,  |n: &NonZeroI128| n.get()  }
