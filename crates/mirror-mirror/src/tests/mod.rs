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
fn box_t_is_reflect<T>(t: Box<T>)
where
    T: Reflect,
{
    let _ = t.as_reflect();
}
