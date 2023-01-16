use macaw::ColorRgba8;
use mirror_mirror_macros::__private_derive_reflect_foreign;

__private_derive_reflect_foreign! {
    #[reflect(crate_name(crate))]
    pub struct ColorRgba8(pub [u8; 4]);
}
