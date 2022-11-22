use proc_macro2::TokenStream;
use quote::quote_spanned;
use syn::{spanned::Spanned, DeriveInput};

mod enum_;
mod struct_named;
mod tuple_struct;

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
            syn::Fields::Unnamed(unnamed) => tuple_struct::expand(ident, unnamed)?,
            // bevy_reflect only implements `Struct` for unit structs, not `TupleStruct`
            // so lets just do the same here
            syn::Fields::Unit => struct_named::expand(
                ident,
                syn::FieldsNamed {
                    brace_token: Default::default(),
                    named: Default::default(),
                },
            )?,
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
            #[allow(unused_imports)]
            use mirror_mirror::*;
            #[allow(unused_imports)]
            use mirror_mirror::__private::*;

            #tokens

            impl From<#ident> for Value {
                fn from(data: #ident) -> Value {
                    data.to_value()
                }
            }

            fn trait_bounds()
            where
                #ident: std::clone::Clone + std::fmt::Debug,
            {}
        };
    })
}
