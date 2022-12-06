use proc_macro2::TokenStream;
use quote::quote_spanned;
use syn::spanned::Spanned;
use syn::DeriveInput;
use syn::ImplGenerics;
use syn::TypeGenerics;
use syn::WhereClause;

mod attrs;
mod enum_;
mod struct_named;
mod tuple_struct;

struct Generics<'a> {
    impl_generics: ImplGenerics<'a>,
    type_generics: TypeGenerics<'a>,
    where_clause: Option<&'a WhereClause>,
}

pub(crate) fn expand(item: DeriveInput) -> syn::Result<TokenStream> {
    let (impl_generics, type_generics, where_clause) = item.generics.split_for_impl();
    let generics = Generics {
        impl_generics,
        type_generics,
        where_clause,
    };

    let ident = &item.ident;
    let span = item.span();
    let attrs = attrs::ItemAttrs::parse(&item.attrs)?;
    let crate_name = attrs.crate_name.clone();

    check_for_known_unsupported_types(&item)?;

    let tokens = match item.data {
        syn::Data::Struct(data) => match data.fields {
            syn::Fields::Named(named) => struct_named::expand(ident, named, attrs, &generics)?,
            syn::Fields::Unnamed(unnamed) => {
                tuple_struct::expand(ident, unnamed, attrs, &generics)?
            }
            // bevy_reflect only implements `Struct` for unit structs, not `TupleStruct`
            // so lets just do the same here
            syn::Fields::Unit => struct_named::expand(
                ident,
                syn::FieldsNamed {
                    brace_token: Default::default(),
                    named: Default::default(),
                },
                attrs,
                &generics,
            )?,
        },
        syn::Data::Enum(enum_) => enum_::expand(ident, enum_, attrs, &generics)?,
        syn::Data::Union(_) => {
            return Err(syn::Error::new(
                span,
                "`#[derive(Reflect)]` doesn't support unions",
            ))
        }
    };

    let Generics {
        impl_generics,
        type_generics,
        where_clause,
    } = generics;

    Ok(quote_spanned! {span=>
        #[allow(
            clippy::implicit_clone,
            clippy::redundant_clone,
            clippy::clone_on_copy,
            unused_variables,
        )]
        const _: () = {

            #[allow(unused_imports)]
            use #crate_name::*;
            #[allow(unused_imports)]
            use #crate_name::__private::*;

            #tokens

            impl #impl_generics From<#ident #type_generics> for Value #where_clause {
                fn from(data: #ident #type_generics) -> Value {
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
