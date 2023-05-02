//! Tame versions of existing containers including trait implementations focused on game development usecases.
//!
//! ## Provided Collections
//!
//! - [`UnorderedMap`], useful when you want a very fast general-purpose key-value map with no order and
//! generous trait implementations, and you plan to do random access lookup by key more frequently than iteration
//! of the contained elements.
//! - [`OrderedMap`], useful when you want a key-value map including a set order of element pairs, or when
//! you plan to iterate over the contained elements more frequently than you do random access lookup by key.
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
#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![cfg_attr(test, allow(clippy::float_cmp))]

/// Provides a fast, general purpose, key-value map with **no** defined order of elements
pub mod unordered_map;
#[doc(inline)]
pub use unordered_map::UnorderedMap;

/// Provides a fast, general purpose, key-value map **with** a defined order of elements.
pub mod ordered_map;
#[doc(inline)]
pub use ordered_map::OrderedMap;

#[cfg(test)]
mod tests;

pub(crate) static STATIC_RANDOM_STATE: ahash::RandomState = ahash::RandomState::with_seeds(
    0x86c11a44c63f4f2f,
    0xaf04d821054d02b3,
    0x98f0a276c462acc1,
    0xe2d6368e09c9c079,
);
