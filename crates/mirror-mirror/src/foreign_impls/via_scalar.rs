use core::num::NonZeroI128;
use core::num::NonZeroI16;
use core::num::NonZeroI32;
use core::num::NonZeroI64;
use core::num::NonZeroI8;
use core::num::NonZeroU128;
use core::num::NonZeroU16;
use core::num::NonZeroU32;
use core::num::NonZeroU64;
use core::num::NonZeroU8;
use core::num::NonZeroUsize;
use core::time::Duration;

macro_rules! impl_reflect_via_scalar {
    ($ty:ty, $via_ty:ty, $get_fn:expr, $new_fn:expr $(,)?) => {
        const _: () = {
            use $crate::__private::*;

            impl DescribeType for $ty {
                fn build(graph: &mut TypeGraph) -> NodeId {
                    graph.get_or_build_node_with::<Self, _>(|graph| {
                        OpaqueNode::new::<Self>(Default::default(), graph)
                    })
                }
            }

            impl Reflect for $ty {
                trivial_reflect_methods!();

                #[allow(clippy::redundant_closure_call)]
                fn reflect_owned(self: Box<Self>) -> ReflectOwned {
                    ReflectOwned::Scalar(ScalarOwned::from($get_fn(&*self)))
                }

                #[allow(clippy::redundant_closure_call)]
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

                #[allow(clippy::redundant_closure_call)]
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
                        <$via_ty>::from_reflect(reflect)
                            .and_then(|value| $new_fn(value).into_option())
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

impl_reflect_via_scalar! { NonZeroUsize, usize, |n: &NonZeroUsize| n.get(), Self::new }
impl_reflect_via_scalar! { NonZeroU8,    u8,    |n: &NonZeroU8| n.get(),    Self::new }
impl_reflect_via_scalar! { NonZeroU16,   u16,   |n: &NonZeroU16| n.get(),   Self::new }
impl_reflect_via_scalar! { NonZeroU32,   u32,   |n: &NonZeroU32| n.get(),   Self::new }
impl_reflect_via_scalar! { NonZeroU64,   u64,   |n: &NonZeroU64| n.get(),   Self::new }
impl_reflect_via_scalar! { NonZeroU128,  u128,  |n: &NonZeroU128| n.get(),  Self::new }
impl_reflect_via_scalar! { NonZeroI8,    i8,    |n: &NonZeroI8| n.get(),    Self::new }
impl_reflect_via_scalar! { NonZeroI16,   i16,   |n: &NonZeroI16| n.get(),   Self::new }
impl_reflect_via_scalar! { NonZeroI32,   i32,   |n: &NonZeroI32| n.get(),   Self::new }
impl_reflect_via_scalar! { NonZeroI64,   i64,   |n: &NonZeroI64| n.get(),   Self::new }
impl_reflect_via_scalar! { NonZeroI128,  i128,  |n: &NonZeroI128| n.get(),  Self::new }

impl_reflect_via_scalar! { Duration, f32, |d: &Duration| d.as_secs_f32(), Self::from_secs_f32 }

trait IntoOption<T> {
    fn into_option(self) -> Option<T>;
}

impl<T> IntoOption<T> for Option<T> {
    fn into_option(self) -> Option<T> {
        self
    }
}

impl<T> IntoOption<T> for T {
    fn into_option(self) -> Option<T> {
        Some(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DescribeType;

    #[test]
    fn keeps_type_name() {
        assert_eq!(
            <NonZeroI8 as DescribeType>::type_descriptor()
                .get_type()
                .type_name(),
            "core::num::nonzero::NonZero<i8>"
        );

        assert_eq!(
            <Duration as DescribeType>::type_descriptor()
                .get_type()
                .type_name(),
            "core::time::Duration"
        );
    }
}
