use proc_macro2::TokenStream;
use quote::quote_spanned;
use syn::{spanned::Spanned, DeriveInput};

mod enum_;
mod struct_named;

pub(crate) fn expand(item: DeriveInput) -> syn::Result<TokenStream> {
    if !item.generics.params.is_empty() || item.generics.where_clause.is_some() {
        return Err(syn::Error::new_spanned(
            &item.generics,
            "`#[derive(Reflect)]` doesn't support generics",
        ));
    }

    let ident = &item.ident;

    let span = item.span();

    let tokens = match item.data {
        syn::Data::Struct(data) => match data.fields {
            syn::Fields::Named(named) => struct_named::expand(ident, named)?,
            syn::Fields::Unnamed(_) => {
                return Err(syn::Error::new(
                    span,
                    "`#[derive(Reflect)]` doesn't support tuple structs",
                ))
            }
            syn::Fields::Unit => {
                return Err(syn::Error::new(
                    span,
                    "`#[derive(Reflect)]` doesn't support unit structs",
                ))
            }
        },
        syn::Data::Enum(enum_) => enum_::expand(ident, enum_)?,
        syn::Data::Union(_) => {
            return Err(syn::Error::new(
                span,
                "`#[derive(Reflect)]` doesn't support unions",
            ))
        }
    };

    Ok(quote_spanned! {span=>
        const _: () = {
            #tokens
        };
    })
}
