use crate as mirror_mirror;
use crate::Reflect;

mod enum_;
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

    #[derive(Reflect, Debug, Clone)]
    struct A {
        a: String,
        b: Vec<B>,
    }

    #[derive(Reflect, Debug, Clone)]
    enum B {
        C(C),
        D { d: D },
    }

    #[derive(Reflect, Debug, Clone)]
    struct C(String, i32, Vec<bool>);

    #[derive(Reflect, Debug, Clone)]
    struct D;
}
