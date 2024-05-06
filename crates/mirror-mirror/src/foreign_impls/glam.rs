use glam::{Mat3, Mat4, Vec2, Vec3, Vec4};
use mirror_mirror_macros::__private_derive_reflect_foreign;

mod mat2;
mod quat;
mod vec4;

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

__private_derive_reflect_foreign! {
    #[reflect(crate_name(crate))]
    pub struct Mat3 {
        pub x_axis: Vec3,
        pub y_axis: Vec3,
        pub z_axis: Vec3,
    }
}

__private_derive_reflect_foreign! {
    #[reflect(crate_name(crate))]
    pub struct Mat4 {
        pub x_axis: Vec4,
        pub y_axis: Vec4,
        pub z_axis: Vec4,
        pub w_axis: Vec4,
    }
}
