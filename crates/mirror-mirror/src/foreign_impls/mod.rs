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
        T: FromReflect + DescribeType,
    {
        None,
        Some(T),
    }
}

__private_derive_reflect_foreign! {
    #[reflect(opt_out(Clone, Debug), crate_name(crate))]
    enum Result<T, E>
    where
        T: FromReflect + DescribeType,
        E: FromReflect + DescribeType,
    {
        Ok(T),
        Err(E),
    }
}

__private_derive_reflect_foreign! {
    #[reflect(opt_out(Clone, Debug), crate_name(crate))]
    struct Range<Idx>
    where
        Idx: FromReflect + DescribeType,
    {
        start: Idx,
        end: Idx,
    }
}

__private_derive_reflect_foreign! {
    #[reflect(opt_out(Clone, Debug), crate_name(crate))]
    struct RangeFrom<Idx>
    where
        Idx: FromReflect + DescribeType,
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
        Idx: FromReflect + DescribeType,
    {
        end: Idx,
    }
}

__private_derive_reflect_foreign! {
    #[reflect(opt_out(Clone, Debug), crate_name(crate))]
    struct RangeTo<Idx>
    where
        Idx: FromReflect + DescribeType,
    {
        end: Idx,
    }
}

#[cfg(feature = "glam")]
mod glam_impls {
    use glam::{Mat3, Vec2, Vec3};
    use mirror_mirror_macros::__private_derive_reflect_foreign;

    __private_derive_reflect_foreign! {
        #[reflect(crate_name(crate))]
        pub struct Vec2 {
            pub x: f32,
            pub y: f32,
        }
    }

    __private_derive_reflect_foreign! {
        #[reflect(crate_name(crate))]
        pub struct Vec3 {
            pub x: f32,
            pub y: f32,
            pub z: f32,
        }
    }

    // `Vec4`, `Quat`, and `Mat2` are left out because glam uses bad hacks which changes the struct
    // definitions for different architectures (simd vs no simd) and cargo features. So we'd have
    // to use the same hacks in mirror-mirror which I'd like to avoid.

    // `Mat4` is left out because it contains `Vec4` which we don't support.

    __private_derive_reflect_foreign! {
        #[reflect(crate_name(crate))]
        pub struct Mat3 {
            pub x_axis: Vec3,
            pub y_axis: Vec3,
            pub z_axis: Vec3,
        }
    }
}

#[cfg(feature = "macaw")]
mod macaw_impls {
    use macaw::ColorRgba8;
    use mirror_mirror_macros::__private_derive_reflect_foreign;

    __private_derive_reflect_foreign! {
        #[reflect(crate_name(crate))]
        pub struct ColorRgba8(pub [u8; 4]);
    }
}
