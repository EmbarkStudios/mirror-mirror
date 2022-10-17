use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::parse::Parse;

mod derive_reflect;

#[proc_macro_derive(Reflect)]
pub fn derive_reflect(item: TokenStream) -> TokenStream {
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
            ::std::stringify!(#token)
        })
    }
}
