use proc_macro2::TokenStream;
use quote::quote_spanned;
use syn::spanned::Spanned;
use syn::DeriveInput;

mod attrs;
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
    let attrs = attrs::ItemAttrs::parse(&item.attrs)?;

    check_for_known_unsupported_types(&item)?;

    let tokens = match item.data {
        syn::Data::Struct(data) => match data.fields {
            syn::Fields::Named(named) => struct_named::expand(ident, named, attrs)?,
            syn::Fields::Unnamed(unnamed) => tuple_struct::expand(ident, unnamed, attrs)?,
            // bevy_reflect only implements `Struct` for unit structs, not `TupleStruct`
            // so lets just do the same here
            syn::Fields::Unit => struct_named::expand(
                ident,
                syn::FieldsNamed {
                    brace_token: Default::default(),
                    named: Default::default(),
                },
                attrs,
            )?,
        },
        syn::Data::Enum(enum_) => enum_::expand(ident, enum_, attrs)?,
        syn::Data::Union(_) => {
            return Err(syn::Error::new(
                span,
                "`#[derive(Reflect)]` doesn't support unions",
            ))
        }
    };

    Ok(quote_spanned! {span=>
        #[allow(clippy::implicit_clone)]
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
        };
    })
}

fn check_for_known_unsupported_types(item: &DeriveInput) -> syn::Result<()> {
    #[derive(Default)]
    struct Visitor(Option<syn::Error>);

    impl<'ast> syn::visit::Visit<'ast> for Visitor {
        fn visit_ident(&mut self, i: &'ast proc_macro2::Ident) {
            if i == "HashMap" && self.0.is_none() {
                self.0 = Some(syn::Error::new_spanned(
                    i,
                    "`#[derive(Reflect)]` doesn't support `HashMap`. Use a `BTreeMap` instead.",
                ));
            }
        }
    }

    let mut visitor = Visitor::default();
    syn::visit::visit_derive_input(&mut visitor, item);

    match visitor.0 {
        Some(err) => Err(err),
        None => Ok(()),
    }
}
