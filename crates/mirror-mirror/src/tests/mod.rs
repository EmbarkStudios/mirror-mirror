use crate::{self as mirror_mirror, Reflect};

mod enum_;
mod struct_;
mod tuple;
mod tuple_struct;

#[derive(Reflect)]
#[reflect(!Debug, !Clone)]
#[allow(dead_code)]
struct DebugOptOut;
