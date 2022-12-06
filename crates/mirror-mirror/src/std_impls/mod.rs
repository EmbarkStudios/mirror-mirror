use mirror_mirror_macros::__private_derive_reflect_foreign;
use std::ops::{Range, RangeFrom, RangeFull, RangeTo, RangeToInclusive};

mod array;
mod btree_map;
mod non_zero;
mod vec;

__private_derive_reflect_foreign! {
    #[reflect(opt_out(Clone, Debug), crate_name(crate))]
    enum Option<T>
    where
        T: FromReflect + Typed,
    {
        Some(T),
        None,
    }
}

__private_derive_reflect_foreign! {
    #[reflect(opt_out(Clone, Debug), crate_name(crate))]
    enum Result<T, E>
    where
        T: FromReflect + Typed,
        E: FromReflect + Typed,
    {
        Ok(T),
        Err(E),
    }
}

__private_derive_reflect_foreign! {
    #[reflect(opt_out(Clone, Debug), crate_name(crate))]
    struct Range<Idx>
    where
        Idx: FromReflect + Typed,
    {
        start: Idx,
        end: Idx,
    }
}

__private_derive_reflect_foreign! {
    #[reflect(opt_out(Clone, Debug), crate_name(crate))]
    struct RangeFrom<Idx>
    where
        Idx: FromReflect + Typed,
    {
        start: Idx,
    }
}

__private_derive_reflect_foreign! {
    #[reflect(crate_name(crate))]
    struct RangeFull;
}

__private_derive_reflect_foreign! {
    #[reflect(opt_out(Clone, Debug), crate_name(crate))]
    struct RangeToInclusive<Idx>
    where
        Idx: FromReflect + Typed,
    {
        end: Idx,
    }
}

__private_derive_reflect_foreign! {
    #[reflect(opt_out(Clone, Debug), crate_name(crate))]
    struct RangeTo<Idx>
    where
        Idx: FromReflect + Typed,
    {
        end: Idx,
    }
}
