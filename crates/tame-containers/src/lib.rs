//! Tame versions of existing containers including trait implementations focused on game development usecases.
//!
//! ## Provided Collections
//!
//! - [`LinearMap`], useful when you want a collection that behaves as a key-value map but you will only ever have
//! up to a few tens of elements. For small numbers of elements and small size of contained key/value types, this
//! will likely be faster and take up less memory than other map types, but its random access operations have O(len)
//! complexity rather than O(1) as with hash-based maps.
//! - [`UnorderedMap`], useful when you want a general-purpose key-value map, you plan to do random access
//! lookup by key more frequently than iteration of the contained elements, and you don't care about order of
//! those elements.
//! - [`OrderedMap`], useful when you want a key-value map including a set order of element pairs, or when
//! you plan to iterate over the contained elements more frequently than you do random access lookup by key.
//! 
//! - [`LinearSet`], useful in the same situation as [`LinearMap`] but when you're operating on a set of values rather
//! than a map.
//! - [`UnorderedSet`], useful when you want set-like operations, will do more random access than iteration,
//! and will fill with a medium to high number of elements.
//! - [`OrderedSet`], useful when you want set-like operations and need a defined order of elements, or you
//! plan to iterate over the contained elements more frequently than do random access lookup on them.
//! 
//! ## Usage table
//! 
//! Number of elements | Access pattern | Need defined order | Choose
//! ---|---|---
//! Less than ~128 | Any | Any | `LinearMap`/`LinearSet`
//! More than ~128 | More Random Access | No | [`UnorderedMap`]/[`UnorderedSet`]
//! More than ~128 | More Iteration | No | [`OrderedMap`]/[`OrderedSet`]
//! More than ~128 | Any | Yes | [`OrderedMap`]/[`OrderedSet`]
//! 
//! # Feature flags
//!
//! `tame-containers` uses a set of [feature flags] to optionally reduce the number of dependencies.
//!
//! The following optional features are available:
//!
//! Name | Description | Default?
//! ---|---|---
//! `speedy` | Enables [`speedy`] support for most types | Yes
//! `serde` | Enables [`serde`] support for most types | Yes
//!
//! [`speedy`]: https://crates.io/crates/speedy
//! [`serde`]: https://crates.io/crates/serde

#![warn(
    clippy::all,
    clippy::dbg_macro,
    clippy::todo,
    clippy::empty_enum,
    clippy::enum_glob_use,
    clippy::mem_forget,
    clippy::unused_self,
    clippy::filter_map_next,
    clippy::needless_continue,
    clippy::needless_borrow,
    clippy::match_wildcard_for_single_variants,
    clippy::if_let_mutex,
    clippy::mismatched_target_os,
    clippy::await_holding_lock,
    clippy::match_on_vec_items,
    clippy::imprecise_flops,
    clippy::suboptimal_flops,
    clippy::lossy_float_literal,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::fn_params_excessive_bools,
    clippy::exit,
    clippy::inefficient_to_string,
    clippy::linkedlist,
    clippy::macro_use_imports,
    clippy::option_option,
    clippy::verbose_file_reads,
    clippy::unnested_or_patterns,
    clippy::str_to_string,
    rust_2018_idioms,
    future_incompatible,
    nonstandard_style,
    missing_debug_implementations
)]
#![deny(
    missing_docs,
    unreachable_pub,
    private_in_public,
    rustdoc::broken_intra_doc_links
)]
#![allow(
    elided_lifetimes_in_paths,
    clippy::type_complexity,
    // because speedy
    clippy::not_unsafe_ptr_arg_deref,
)]
#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![cfg_attr(test, allow(clippy::float_cmp))]

/// Provides a fast, general purpose, key-value map with **no** defined order of elements
pub mod unordered_map;
#[doc(inline)]
pub use unordered_map::UnorderedMap;

/// Provides a fast, general purpose, deduplicated set type with **no** defined order of elements
pub mod unordered_set;
#[doc(inline)]
pub use unordered_set::UnorderedSet;

/// Provides a fast, general purpose, key-value map **with** a defined order of elements.
pub mod ordered_map;
#[doc(inline)]
pub use ordered_map::OrderedMap;

/// Provides a fast, general purpose, deduplicated set type **with* a defined order of elements.
pub mod ordered_set;
#[doc(inline)]
pub use ordered_set::OrderedSet;

/// A key-value map specialized for small numbers of elements, implemented by searching linearly in a vector.
pub mod linear_map;
#[doc(inline)]
pub use linear_map::LinearMap;

/// A set specialized for small numbers of elements, implemented by searching linearly in a vector.
pub mod linear_set;
#[doc(inline)]
pub use linear_set::LinearSet;

#[cfg(test)]
mod tests;

pub(crate) static STATIC_RANDOM_STATE: ahash::RandomState = ahash::RandomState::with_seeds(
    0x86c11a44c63f4f2f,
    0xaf04d821054d02b3,
    0x98f0a276c462acc1,
    0xe2d6368e09c9c079,
);
