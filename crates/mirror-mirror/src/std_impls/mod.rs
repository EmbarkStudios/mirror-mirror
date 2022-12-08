use core::ops::Range;
use core::ops::RangeFrom;
use core::ops::RangeFull;
use core::ops::RangeTo;
use core::ops::RangeToInclusive;

use mirror_mirror_macros::__private_derive_reflect_foreign;

mod array;
mod boxed;
mod btree_map;
mod vec;
mod via_scalar;

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
