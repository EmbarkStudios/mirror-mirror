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
    // missing_debug_implementations,
    // missing_docs
)]
#![deny(unreachable_pub, private_in_public)]
#![allow(elided_lifetimes_in_paths, clippy::type_complexity)]
#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![cfg_attr(test, allow(clippy::float_cmp))]

use proc_macro::TokenStream;
use quote::quote;
use quote::ToTokens;
use syn::parse::Parse;

mod derive_reflect;

#[proc_macro_derive(Reflect, attributes(reflect))]
pub fn derive_reflect(item: TokenStream) -> TokenStream {
    expand_with(item, derive_reflect::expand)
}

/// Private API: Do not use!
#[proc_macro]
pub fn __private_derive_reflect_foreign(item: TokenStream) -> TokenStream {
    expand_with(item, derive_reflect::expand)
}

/// Private API: Do not use!
#[proc_macro]
pub fn __private_derive_reflect_foreign_debug(item: TokenStream) -> TokenStream {
    let tokens = expand_with(item, derive_reflect::expand);
    eprintln!("{tokens}");
    tokens
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
        Ok(tokens) => {
            let tokens = quote! { #tokens }.into();
            if std::env::var_os("MIRROR_MIRROR_DEBUG").is_some() {
                eprintln!("{tokens}");
            }
            tokens
        }
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
            ::std::stringify!(#token)
        })
    }
}
