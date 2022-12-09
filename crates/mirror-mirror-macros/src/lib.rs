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
    missing_debug_implementations,
    // missing_docs
)]
#![deny(unreachable_pub, private_in_public)]
#![allow(elided_lifetimes_in_paths, clippy::type_complexity)]
#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![cfg_attr(test, allow(clippy::float_cmp))]

extern crate alloc;

use proc_macro::TokenStream;
use quote::quote;
use quote::ToTokens;
use syn::parse::Parse;

mod derive_reflect;

/// Derive an implementation of `Reflect` and other appropriate traits.
///
/// # Structs
///
/// On structs `#[derive(Reflect)]` will also derive `Struct` and `FromReflect`.
///
/// ```
/// use mirror_mirror::Reflect;
///
/// #[derive(Reflect, Clone, Debug)]
/// struct Foo {
///     a: i32,
///     b: bool,
///     c: String,
/// }
/// ```
///
/// Unit structs are treated as tuple structs with no fields.
///
/// # Tuple structs
///
/// On tuple structs `#[derive(Reflect)]` will also derive `TupleStruct` and `FromReflect`.
///
/// ```
/// use mirror_mirror::Reflect;
///
/// #[derive(Reflect, Clone, Debug)]
/// struct Foo(i32, bool, String);
/// ```
///
/// # Enums
///
/// On enums `#[derive(Reflect)]` will also derive `Enum` and `FromReflect`.
///
/// ```
/// use mirror_mirror::Reflect;
///
/// #[derive(Reflect, Clone, Debug)]
/// enum Foo {
///     A(i32),
///     B { b: bool },
///     C,
/// }
/// ```
///
/// # Options
///
/// ## `opt_out`
///
/// By default types are required to implement `Clone` and `Debug`. You can opt-out of these
/// requirements with `#[reflect(opt_out(Clone, Debug))]`
///
/// ```
/// use mirror_mirror::Reflect;
///
/// #[derive(Reflect)]
/// #[reflect(opt_out(Debug, Clone))]
/// struct Foo(i32);
/// ```
///
/// This changes the implementation of `Reflect::clone_reflect` and `Reflect::debug` to something
/// that works for any type but is less performant.
///
/// You can also opt-out of deriving `FromReflect` so you can provide you own implementation:
///
/// ```
/// use mirror_mirror::{Reflect, FromReflect};
///
/// #[derive(Reflect, Debug, Clone)]
/// #[reflect(opt_out(FromReflect))]
/// struct Foo(i32);
///
/// impl FromReflect for Foo {
///     fn from_reflect(value: &dyn Reflect) -> Option<Self> {
///         Some(Self(*value.downcast_ref::<i32>()?))
///     }
/// }
/// ```
///
/// ## `skip`
///
/// You can exclude fields or variants from being reflected with `#[reflect(skip)]`. The type is
/// required to implement `Default` by the default `FromReflect` implementation.
///
/// ```
/// use mirror_mirror::{Reflect, FromReflect};
///
/// #[derive(Reflect, Debug, Clone)]
/// struct Foo {
///     #[reflect(skip)]
///     not_reflect: NotReflect,
/// }
///
/// #[derive(Reflect, Debug, Clone)]
/// struct Bar(#[reflect(skip)] NotReflect);
///
/// #[derive(Reflect, Debug, Clone)]
/// enum Baz {
///     #[reflect(skip)]
///     OnVariant(NotReflect),
///
///     OnTupleField(#[reflect(skip)] NotReflect),
///
///     OnStructField {
///         #[reflect(skip)]
///         not_reflect: NotReflect,
///     }
/// }
///
/// // A type that isn't compatible with reflection
/// #[derive(Debug, Clone, Default)]
/// struct NotReflect;
/// ```
///
/// ## `from_reflect_with`
///
/// You can override `FromReflect` for a single field by specifying a function to do the
/// conversion:
///
/// ```
/// use mirror_mirror::{Reflect, FromReflect};
///
/// #[derive(Reflect, Debug, Clone)]
/// struct Foo {
///     #[reflect(from_reflect_with(n_from_reflect))]
///     n: i32,
/// }
///
/// fn n_from_reflect(field: &dyn Reflect) -> Option<i32> {
///     Some(*field.downcast_ref::<i32>()?)
/// }
/// ```
///
/// ## `meta`
///
/// Metadata associated with types or enum variants be added with `#[reflect(meta(...))]`
///
/// ```
/// use mirror_mirror::{
///     Reflect,
///     key_path,
///     key_path::GetTypePath,
///     FromReflect,
///     type_info::{GetMeta, Typed},
/// };
///
/// #[derive(Reflect, Debug, Clone)]
/// #[reflect(meta(
///     // a comma separated list of `key = value` pairs.
///     //
///     // `key` must be an identifier and `value` can be anything that
///     // implements `Reflect`
///     item_key = "item value",
/// ))]
/// struct Foo {
///     #[reflect(meta(field_key = 1337))]
///     n: i32,
/// }
///
/// // Access the metadata through the type information
/// let type_info = <Foo as Typed>::type_info();
///
/// assert_eq!(
///     type_info.get_meta::<String>("item_key").unwrap(),
///     "item value",
/// );
///
/// assert_eq!(
///     type_info
///         .as_struct()
///         .unwrap()
///         .field_type("n")
///         .unwrap()
///         .get_meta::<i32>("field_key")
///         .unwrap(),
///     &1337,
/// );
/// ```
///
/// ## `crate_name`
///
/// You can specify a "use path" for `mirror_mirror` with `crate_name`. This is useful if you're
/// using a library that re-exports `mirror_mirror`'s derive macro:
///
/// ```
/// # use mirror_mirror as some_library;
/// use some_library::Reflect;
///
/// #[derive(Reflect, Debug, Clone)]
/// #[reflect(crate_name(some_library))]
/// struct Foo {
///     n: i32,
/// }
/// ```
///
/// This causes the macro generate paths like `some_library::FromReflect`.
///
/// [`Reflect`]: crate::Reflect
#[proc_macro_derive(Reflect, attributes(reflect))]
pub fn derive_reflect(item: TokenStream) -> TokenStream {
    expand_with(item, derive_reflect::expand)
}

/// Private API: Do not use!
#[proc_macro]
#[doc(hidden)]
pub fn __private_derive_reflect_foreign(item: TokenStream) -> TokenStream {
    expand_with(item, derive_reflect::expand)
}

fn expand_with<F, I, K>(input: TokenStream, f: F) -> TokenStream
where
    F: FnOnce(I) -> syn::Result<K>,
    I: Parse,
    K: ToTokens,
{
    expand(syn::parse(input).and_then(f))
}

fn expand<T>(result: syn::Result<T>) -> TokenStream
where
    T: ToTokens,
{
    match result {
        Ok(tokens) => quote! { #tokens }.into(),
        Err(err) => err.into_compile_error().into(),
    }
}

fn stringify<T>(token: T) -> Stringify<T> {
    Stringify(token)
}

struct Stringify<T>(T);

impl<T: ToTokens> ToTokens for Stringify<T>
where
    T: ToTokens,
{
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let token = &self.0;
        tokens.extend(quote::quote! {
            ::core::stringify!(#token)
        })
    }
}
