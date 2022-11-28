use crate as mirror_mirror;
use crate::Reflect;

mod enum_;
mod list;
mod map;
mod struct_;
mod tuple;
mod tuple_struct;

#[derive(Reflect)]
#[reflect(!Debug, !Clone)]
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
