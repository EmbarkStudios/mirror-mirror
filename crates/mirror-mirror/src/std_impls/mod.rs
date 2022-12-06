use mirror_mirror_macros::__private_derive_reflect_foreign;

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
