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
