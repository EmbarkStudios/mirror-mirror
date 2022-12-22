use crate::Reflect;

mod array;
mod enum_;
mod key_path;
mod list;
mod map;
mod meta;
mod simple_type_name;
mod struct_;
mod tuple;
mod tuple_struct;
mod type_info;
mod value;

#[derive(Reflect)]
#[reflect(crate_name(crate), opt_out(Debug, Clone))]
#[allow(dead_code)]
struct DebugOptOut;

#[derive(Reflect)]
#[reflect(crate_name(crate), opt_out(Debug, Clone))]
#[allow(dead_code)]
struct ContainsBoxed(Box<f32>);

mod complex_types {
    #![allow(dead_code)]

    use alloc::collections::BTreeMap;

    use crate::Reflect;

    #[derive(Reflect, Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
    #[reflect(crate_name(crate))]
    struct A {
        a: String,
        b: Vec<B>,
        d: BTreeMap<B, Vec<A>>,
    }

    #[derive(Reflect, Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
    #[reflect(crate_name(crate))]
    enum B {
        C(C),
        D { d: D },
    }

    #[derive(Reflect, Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
    #[reflect(crate_name(crate))]
    struct C(String, i32, Vec<bool>);

    #[derive(Reflect, Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
    #[reflect(crate_name(crate))]
    struct D;
}

mod skip {
    #![allow(dead_code)]

    use super::*;

    #[derive(Reflect, Debug, Clone)]
    #[reflect(crate_name(crate))]
    struct TestStruct {
        #[reflect(skip)]
        not_reflect: NotReflect,
    }

    #[derive(Reflect, Debug, Clone)]
    #[reflect(crate_name(crate))]
    struct TestTupleStruct(#[reflect(skip)] NotReflect);

    #[derive(Reflect, Debug, Clone)]
    #[reflect(crate_name(crate))]
    #[allow(clippy::enum_variant_names)]
    enum TestEnum {
        #[reflect(skip)]
        SkipStructVariant {
            not_reflect: NotReflect,
        },
        SkipStructField {
            #[reflect(skip)]
            not_reflect: NotReflect,
        },
        #[reflect(skip)]
        SkipTupleVariant(NotReflect),
        SkipTupleField(#[reflect(skip)] NotReflect),
        #[reflect(skip)]
        SkipUnitVariant,
    }

    #[derive(Debug, Clone, Default)]
    struct NotReflect;
}

mod option_f32 {
    #![allow(dead_code)]

    use super::*;

    #[derive(Debug, Clone, Reflect)]
    #[reflect(crate_name(crate))]
    struct Foo {
        maybe_float: Option<f32>,
        maybe_string: Option<String>,
    }
}

mod derive_foreign {
    #![allow(dead_code)]

    use mirror_mirror_macros::*;

    use crate::FromReflect;
    use crate::Typed;

    enum Foo<A, B>
    where
        A: FromReflect + Typed,
        B: FromReflect + Typed,
    {
        Struct { a: A },
        Tuple(B),
        Unit,
    }

    __private_derive_reflect_foreign! {
        #[reflect(opt_out(Clone, Debug), crate_name(crate))]
        enum Foo<A, B>
        where
            A: FromReflect + Typed,
            B: FromReflect + Typed,
        {
            Struct { a: A },
            Tuple(B),
            Unit,
        }
    }

    struct Bar<A, B>
    where
        A: FromReflect + Typed,
        B: FromReflect + Typed,
    {
        a: A,
        b: B,
    }

    __private_derive_reflect_foreign! {
        #[reflect(opt_out(Clone, Debug), crate_name(crate))]
        struct Bar<A, B>
        where
            A: FromReflect + Typed,
            B: FromReflect + Typed,
        {
            a: A,
            b: B,
        }
    }

    struct Baz<A, B>(A, B)
    where
        A: FromReflect + Typed,
        B: FromReflect + Typed;

    __private_derive_reflect_foreign! {
        #[reflect(opt_out(Clone, Debug), crate_name(crate))]
        struct Baz<A, B>(A, B)
        where
            A: FromReflect + Typed,
            B: FromReflect + Typed;
    }

    struct Qux;

    __private_derive_reflect_foreign! {
        #[reflect(opt_out(Clone, Debug), crate_name(crate))]
        struct Qux;
    }
}

mod from_reflect_opt_out {
    #![allow(warnings)]

    use super::*;
    use crate::FromReflect;

    #[derive(Reflect, Debug, Clone, Copy, PartialEq)]
    #[reflect(crate_name(crate), opt_out(FromReflect))]
    struct Percentage(f32);

    impl FromReflect for Percentage {
        fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
            if let Some(this) = reflect.downcast_ref::<Self>() {
                Some(*this)
            } else if let Some(value) = f32::from_reflect(reflect) {
                Some(Self(value.clamp(0.0, 100.0)))
            } else if let Some(value) = f64::from_reflect(reflect) {
                Some(Self((value as f32).clamp(0.0, 100.0)))
            } else {
                None
            }
        }
    }

    #[test]
    fn works() {
        assert_eq!(
            Percentage::from_reflect(&Percentage(10.0)).unwrap(),
            Percentage(10.0)
        );

        assert_eq!(Percentage::from_reflect(&10.0).unwrap(), Percentage(10.0));

        assert_eq!(
            Percentage::from_reflect(&1337.0).unwrap(),
            Percentage(100.0)
        );
    }

    #[derive(Reflect, Debug, Clone)]
    #[reflect(crate_name(crate), opt_out(FromReflect))]
    struct B {
        n: f32,
    }

    impl FromReflect for B {
        fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
            None
        }
    }

    #[derive(Reflect, Debug, Clone)]
    #[reflect(crate_name(crate), opt_out(FromReflect))]
    enum C {
        A(f32),
    }

    impl FromReflect for C {
        fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
            None
        }
    }
}

mod from_reflect_with {
    #![allow(warnings)]

    use super::*;
    use crate::FromReflect;

    #[derive(Reflect, Debug, Clone, Copy, PartialEq)]
    #[reflect(crate_name(crate))]
    struct A {
        #[reflect(from_reflect_with(clamp_ratio))]
        a: f32,
    }

    #[derive(Reflect, Debug, Clone, Copy, PartialEq)]
    #[reflect(crate_name(crate))]
    struct B(#[reflect(from_reflect_with(clamp_ratio))] f32);

    #[derive(Reflect, Debug, Clone, Copy, PartialEq)]
    #[reflect(crate_name(crate))]
    enum C {
        C(#[reflect(from_reflect_with(clamp_ratio))] f32),
        D {
            #[reflect(from_reflect_with(clamp_ratio))]
            d: f32,
        },
    }

    fn clamp_ratio(ratio: &dyn Reflect) -> Option<f32> {
        Some(ratio.downcast_ref::<f32>()?.clamp(0.0, 1.0))
    }

    #[test]
    fn works() {
        assert_eq!(A::from_reflect(&A { a: 100.0 }).unwrap(), A { a: 1.0 });
        assert_eq!(A::from_reflect(&A { a: -100.0 }).unwrap(), A { a: 0.0 });

        assert_eq!(B::from_reflect(&B(100.0)).unwrap(), B(1.0));
        assert_eq!(B::from_reflect(&B(-100.0)).unwrap(), B(0.0));

        assert_eq!(C::from_reflect(&C::C(100.0)).unwrap(), C::C(1.0));
        assert_eq!(C::from_reflect(&C::C(-100.0)).unwrap(), C::C(0.0));

        assert_eq!(
            C::from_reflect(&C::D { d: 100.0 }).unwrap(),
            C::D { d: 1.0 }
        );
        assert_eq!(
            C::from_reflect(&C::D { d: -100.0 }).unwrap(),
            C::D { d: 0.0 }
        );
    }
}
