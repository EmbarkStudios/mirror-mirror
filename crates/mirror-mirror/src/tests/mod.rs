use crate as mirror_mirror;
use crate::Reflect;

mod enum_;
mod key_path;
mod list;
mod map;
mod meta;
mod struct_;
mod tuple;
mod tuple_struct;

#[derive(Reflect)]
#[reflect(opt_out(Debug, Clone))]
#[allow(dead_code)]
struct DebugOptOut;

#[allow(warnings)]
fn box_t_is_reflectable<T>(t: Box<T>)
where
    T: Reflect,
{
    let _ = t.as_reflect();
}

mod complex_types {
    #![allow(dead_code)]

    use crate as mirror_mirror;
    use crate::Reflect;
    use std::collections::BTreeMap;

    #[derive(Reflect, Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
    struct A {
        a: String,
        b: Vec<B>,
        d: BTreeMap<B, Vec<A>>,
    }

    #[derive(Reflect, Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
    enum B {
        C(C),
        D { d: D },
    }

    #[derive(Reflect, Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
    struct C(String, i32, Vec<bool>);

    #[derive(Reflect, Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
    struct D;
}

mod skip {
    #![allow(dead_code)]

    use super::*;

    #[derive(Reflect, Debug, Clone)]
    struct TestStruct {
        #[reflect(skip)]
        not_reflect: NotReflect,
    }

    #[derive(Reflect, Debug, Clone)]
    struct TestTupleStruct(#[reflect(skip)] NotReflect);

    #[derive(Reflect, Debug, Clone)]
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
    struct Foo {
        maybe_float: Option<f32>,
        maybe_string: Option<String>,
    }
}

mod derive_foreign {
    #![allow(dead_code)]

    use crate::{FromReflect, Typed};
    use mirror_mirror_macros::*;

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
