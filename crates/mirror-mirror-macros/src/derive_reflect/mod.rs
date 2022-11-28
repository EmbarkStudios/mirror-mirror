use proc_macro2::TokenStream;
use quote::quote;
use quote::quote_spanned;
use syn::parse::ParseStream;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::Attribute;
use syn::DeriveInput;
use syn::Token;

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
    let attrs = ItemAttrs::parse(&item.attrs)?;

    check_for_known_unsupported_type(&item)?;

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

fn check_for_known_unsupported_type(item: &DeriveInput) -> syn::Result<()> {
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

#[derive(Debug, Default, Clone, Copy)]
struct ItemAttrs {
    debug_opt_out: bool,
    clone_opt_out: bool,
}

impl ItemAttrs {
    fn parse(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut reflect_attrs = attrs
            .iter()
            .filter(|attr| attr.path.is_ident("reflect"))
            .peekable();

        let Some(attr) = reflect_attrs.next() else { return Ok(Default::default()) };

        if let Some(next) = reflect_attrs.peek() {
            return Err(syn::Error::new_spanned(
                next,
                "Can only have one `#[reflect(...)]` attribute",
            ));
        }

        let punctuated = attr.parse_args_with(|input: ParseStream<'_>| {
            Punctuated::<_, Token![,]>::parse_terminated_with(input, |input| {
                input.parse::<Token![!]>()?;

                let mut debug_opt_out = false;
                let mut clone_opt_out = false;

                let lh = input.lookahead1();
                if lh.peek(kw::Debug) {
                    input.parse::<kw::Debug>()?;
                    debug_opt_out = true;
                } else if lh.peek(kw::Clone) {
                    input.parse::<kw::Clone>()?;
                    clone_opt_out = true;
                } else {
                    return Err(lh.error());
                }

                Ok((debug_opt_out, clone_opt_out))
            })
        })?;

        let (debug_opt_out, clone_opt_out) = punctuated
            .iter()
            .fold((false, false), |acc, &(debug, clone)| {
                (acc.0 || debug, acc.1 || clone)
            });

        Ok(ItemAttrs {
            debug_opt_out,
            clone_opt_out,
        })
    }

    fn fn_debug_tokens(&self) -> TokenStream {
        if self.debug_opt_out {
            quote! {
                fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    write!(f, "{}", std::any::type_name::<Self>())
                }
            }
        } else {
            quote! {
                fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    if f.alternate() {
                        write!(f, "{:#?}", self)
                    } else {
                        write!(f, "{:?}", self)
                    }
                }
            }
        }
    }

    fn fn_clone_reflect_tokens(&self) -> TokenStream {
        if self.clone_opt_out {
            quote! {
                fn clone_reflect(&self) -> Box<dyn Reflect> {
                    let value = self.to_value();
                    Box::new(Self::from_reflect(&value).unwrap())
                }
            }
        } else {
            quote! {
                fn clone_reflect(&self) -> Box<dyn Reflect> {
                    Box::new(self.clone())
                }
            }
        }
    }
}

mod kw {
    syn::custom_keyword!(Debug);
    syn::custom_keyword!(Clone);
}
