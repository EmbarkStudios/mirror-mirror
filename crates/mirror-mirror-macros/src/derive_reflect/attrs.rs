use std::collections::HashMap;

use proc_macro2::Ident;
use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::ParseStream;
use syn::punctuated::Punctuated;
use syn::Attribute;
use syn::DataEnum;
use syn::Field;
use syn::FieldsNamed;
use syn::FieldsUnnamed;
use syn::Token;
use syn::Variant;

mod kw {
    syn::custom_keyword!(Debug);
    syn::custom_keyword!(Clone);
    syn::custom_keyword!(skip);
}

#[derive(Debug, Default, Clone, Copy)]
pub(super) struct ItemAttrs {
    debug_opt_out: bool,
    clone_opt_out: bool,
}

impl ItemAttrs {
    pub(super) fn parse(attrs: &[Attribute]) -> syn::Result<Self> {
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

    pub(super) fn fn_debug_tokens(&self) -> TokenStream {
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

    pub(super) fn fn_clone_reflect_tokens(&self) -> TokenStream {
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

pub(super) struct AttrsDatabase<T> {
    map: HashMap<T, FieldAttrs>,
}

impl AttrsDatabase<Ident> {
    pub(super) fn new_from_named(fields: &FieldsNamed) -> syn::Result<Self> {
        let map = fields
            .named
            .iter()
            .map(|field| {
                let attrs = FieldAttrs::parse(&field.attrs)?;
                Ok((field.ident.clone().unwrap(), attrs))
            })
            .collect::<syn::Result<HashMap<_, _>>>()?;

        Ok(Self { map })
    }

    pub(super) fn new_from_enum_for_variants(enum_: &DataEnum) -> syn::Result<Self> {
        let map = enum_
            .variants
            .iter()
            .map(|variant| {
                let attrs = FieldAttrs::parse(&variant.attrs)?;
                Ok((variant.ident.clone(), attrs))
            })
            .collect::<syn::Result<HashMap<_, _>>>()?;

        Ok(Self { map })
    }

    pub(super) fn filter_out_skipped_named(&self) -> impl Fn(&&Field) -> bool + '_ {
        move |field| !self.skip(field.ident.as_ref().unwrap())
    }

    pub(super) fn filter_out_skipped_variant(&self) -> impl Fn(&&Variant) -> bool + '_ {
        move |variant| !self.skip(&variant.ident)
    }
}

impl AttrsDatabase<usize> {
    pub(super) fn new_from_unnamed(fields: &FieldsUnnamed) -> syn::Result<Self> {
        let map = fields
            .unnamed
            .iter()
            .enumerate()
            .map(|(index, field)| {
                let attrs = FieldAttrs::parse(&field.attrs)?;
                Ok((index, attrs))
            })
            .collect::<syn::Result<HashMap<_, _>>>()?;

        Ok(Self { map })
    }

    pub(super) fn filter_out_skipped_unnamed<T>(&self) -> impl Fn(&(usize, T)) -> bool + '_ {
        move |(index, _)| !self.skip(index)
    }
}

impl<T> AttrsDatabase<T>
where
    T: std::hash::Hash + Eq,
{
    pub(super) fn skip(&self, key: &T) -> bool {
        self.map
            .get(key)
            .map(|attrs| attrs.skip)
            .unwrap_or_default()
    }
}

#[derive(Debug, Default)]
struct FieldAttrs {
    skip: bool,
}

impl FieldAttrs {
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

        attr.parse_args_with(|input: ParseStream<'_>| {
            let mut field_attrs = Self::default();

            while !input.is_empty() {
                let lh = input.lookahead1();
                if lh.peek(kw::skip) {
                    input.parse::<kw::skip>()?;
                    field_attrs.skip = true;
                } else {
                    return Err(lh.error());
                }

                let _ = input.parse::<Token![,]>();
            }

            Ok(field_attrs)
        })
    }
}
