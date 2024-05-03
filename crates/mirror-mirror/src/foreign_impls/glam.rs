use glam::{Mat3, Vec2, Vec3};
use mirror_mirror_macros::__private_derive_reflect_foreign;

mod vec4;
mod mat2;

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

// TODO(david): `Quat`, `Mat2`, `Mat4`

__private_derive_reflect_foreign! {
    #[reflect(crate_name(crate))]
    pub struct Mat3 {
        pub x_axis: Vec3,
        pub y_axis: Vec3,
        pub z_axis: Vec3,
    }
}
