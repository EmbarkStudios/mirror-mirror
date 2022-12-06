use proc_macro2::Ident;
use proc_macro2::TokenStream;
use quote::quote;
use std::collections::HashMap;
use syn::parse::ParseStream;
use syn::Attribute;
use syn::Expr;
use syn::Field;
use syn::FieldsNamed;
use syn::FieldsUnnamed;
use syn::Lit;
use syn::LitStr;
use syn::Meta;
use syn::Token;
use syn::UseTree;

mod kw {
    syn::custom_keyword!(Debug);
    syn::custom_keyword!(Clone);
    syn::custom_keyword!(skip);
    syn::custom_keyword!(meta);
    syn::custom_keyword!(opt_out);
    syn::custom_keyword!(crate_name);
}

#[derive(Debug, Clone)]
pub(super) struct ItemAttrs {
    pub(super) debug_opt_out: bool,
    pub(super) clone_opt_out: bool,
    pub(super) crate_name: UseTree,
    meta: HashMap<Ident, Expr>,
    docs: Vec<LitStr>,
}

impl ItemAttrs {
    fn new(docs: Vec<LitStr>) -> Self {
        Self {
            debug_opt_out: Default::default(),
            clone_opt_out: Default::default(),
            meta: Default::default(),
            docs,
            crate_name: syn::parse_quote!(mirror_mirror),
        }
    }

    pub(super) fn parse(attrs: &[Attribute]) -> syn::Result<Self> {
        let docs = parse_docs(attrs);

        let mut reflect_attrs = attrs
            .iter()
            .filter(|attr| attr.path.is_ident("reflect"))
            .peekable();

        let Some(attr) = reflect_attrs.next() else { return Ok(Self::new(docs)) };

        if let Some(next) = reflect_attrs.peek() {
            return Err(syn::Error::new_spanned(
                next,
                "Can only have one `#[reflect(...)]` attribute",
            ));
        }

        attr.parse_args_with(|input: ParseStream<'_>| {
            let mut item_attrs = Self::new(docs);

            while !input.is_empty() {
                let lh = input.lookahead1();

                if lh.peek(kw::opt_out) {
                    input.parse::<kw::opt_out>()?;
                    let content;
                    syn::parenthesized!(content in input);
                    while !content.is_empty() {
                        let lh = content.lookahead1();
                        if lh.peek(kw::Debug) {
                            content.parse::<kw::Debug>()?;
                            item_attrs.debug_opt_out = true;
                        } else if lh.peek(kw::Clone) {
                            content.parse::<kw::Clone>()?;
                            item_attrs.clone_opt_out = true;
                        } else {
                            return Err(lh.error());
                        }

                        let _ = content.parse::<Token![,]>();
                    }
                } else if lh.peek(kw::meta) {
                    input.parse::<kw::meta>()?;
                    let content;
                    syn::parenthesized!(content in input);
                    while !content.is_empty() {
                        let ident = content.parse::<Ident>()?;
                        content.parse::<Token![=]>()?;
                        let expr = content.parse::<Expr>()?;
                        if item_attrs.meta.insert(ident.clone(), expr).is_some() {
                            return Err(syn::Error::new_spanned(
                                &ident,
                                format!("`{ident}` specified more than once"),
                            ));
                        }

                        let _ = content.parse::<Token![,]>();
                    }
                } else if lh.peek(kw::crate_name) {
                    input.parse::<kw::crate_name>()?;
                    let content;
                    syn::parenthesized!(content in input);
                    item_attrs.crate_name = content.parse()?;
                } else {
                    return Err(lh.error());
                }

                let _ = input.parse::<Token![,]>();
            }

            Ok(item_attrs)
        })
    }

    pub(super) fn fn_debug_tokens(&self) -> TokenStream {
        if self.debug_opt_out {
            quote! {
                fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    fn go(value: impl ::core::fmt::Debug, f: &mut ::core::fmt::Formatter<'_>) -> fmt::Result {
                        if f.alternate() {
                            write!(f, "{:#?}", value)
                        } else {
                            write!(f, "{:?}", value)
                        }
                    }

                    match self.reflect_ref() {
                        ReflectRef::Struct(inner) => {
                            let mut f = f.debug_struct(inner.type_name());
                            for (name, value) in inner.fields() {
                                f.field(name, &value as &dyn ::core::fmt::Debug);
                            }
                            f.finish()
                        },
                        ReflectRef::TupleStruct(inner) => {
                            let mut f = f.debug_tuple(inner.type_name());
                            for field in inner.fields() {
                                f.field(&field as &dyn ::core::fmt::Debug);
                            }
                            f.finish()
                        },
                        ReflectRef::Tuple(inner) => {
                            let mut f = f.debug_tuple("");
                            for field in inner.fields() {
                                f.field(&field as &dyn ::core::fmt::Debug);
                            }
                            f.finish()
                        },
                        ReflectRef::Enum(inner) => {
                            match inner.variant_kind() {
                                VariantKind::Struct => {
                                    let mut f = f.debug_struct(inner.variant_name());
                                    for field in inner.fields() {
                                        match field {
                                            VariantField::Struct(name, value) => {
                                                f.field(name, &value as &dyn ::core::fmt::Debug);
                                            }
                                            VariantField::Tuple { .. } => {
                                                unreachable!("unit variant yielded struct field")
                                            }
                                        }
                                    }
                                    f.finish()
                                },
                                VariantKind::Tuple => {
                                    let mut f = f.debug_tuple(inner.variant_name());
                                    for field in inner.fields() {
                                        match field {
                                            VariantField::Struct { .. } => unreachable!("unit variant yielded struct field"),
                                            VariantField::Tuple(value) => {
                                                f.field(&value as &dyn ::core::fmt::Debug);
                                            }
                                        }
                                    }
                                    f.finish()
                                },
                                VariantKind::Unit => write!(f, "{}", inner.variant_name()),
                            }
                        },
                        ReflectRef::Array(inner) => {
                            f.debug_list().entries(inner.iter()).finish()
                        },
                        ReflectRef::List(inner) => {
                            f.debug_list().entries(inner.iter()).finish()
                        },
                        ReflectRef::Map(inner) => {
                            f.debug_map().entries(inner.iter()).finish()
                        },
                        ReflectRef::Scalar(inner) => {
                            match inner {
                                ScalarRef::usize(inner) => go(inner, f),
                                ScalarRef::u8(inner) => go(inner, f),
                                ScalarRef::u16(inner) => go(inner, f),
                                ScalarRef::u32(inner) => go(inner, f),
                                ScalarRef::u64(inner) => go(inner, f),
                                ScalarRef::u128(inner) => go(inner, f),
                                ScalarRef::i8(inner) => go(inner, f),
                                ScalarRef::i16(inner) => go(inner, f),
                                ScalarRef::i32(inner) => go(inner, f),
                                ScalarRef::i64(inner) => go(inner, f),
                                ScalarRef::i128(inner) => go(inner, f),
                                ScalarRef::bool(inner) => go(inner, f),
                                ScalarRef::char(inner) => go(inner, f),
                                ScalarRef::f32(inner) => go(inner, f),
                                ScalarRef::f64(inner) => go(inner, f),
                                ScalarRef::String(inner) => go(inner, f),
                            }
                        },
                        ReflectRef::Opaque(_) => {
                            write!(f, "{}", std::any::type_name::<Self>())
                        }
                    }
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

    pub(super) fn meta(&self) -> TokenStream {
        tokenize_meta(&self.meta)
    }

    pub(super) fn docs(&self) -> TokenStream {
        let docs = &self.docs;
        quote! { &[#(#docs,)*] }
    }
}

fn parse_docs(attrs: &[Attribute]) -> Vec<LitStr> {
    attrs
        .iter()
        .filter(|attr| attr.path.is_ident("doc"))
        .filter_map(|attr| {
            let meta = attr.parse_meta().ok()?;
            let Meta::NameValue(pair) = meta else { return None };
            let Lit::Str(lit_str) = pair.lit else { return None };
            Some(lit_str)
        })
        .collect::<Vec<_>>()
}

fn tokenize_meta(meta: &HashMap<Ident, Expr>) -> TokenStream {
    let pairs = meta.iter().map(|(ident, expr)| {
        quote! {
            (stringify!(#ident).to_owned(), IntoValue::into_value(#expr)),
        }
    });
    quote! {
        HashMap::from([#(#pairs)*])
    }
}

pub(super) struct AttrsDatabase<T> {
    map: HashMap<T, InnerAttrs>,
}

impl AttrsDatabase<Ident> {
    pub(super) fn new_from_named(fields: &FieldsNamed) -> syn::Result<Self> {
        let map = fields
            .named
            .iter()
            .map(|field| {
                let attrs = InnerAttrs::parse(&field.attrs)?;
                Ok((field.ident.clone().unwrap(), attrs))
            })
            .collect::<syn::Result<HashMap<_, _>>>()?;

        Ok(Self { map })
    }

    pub(super) fn filter_out_skipped_named(&self) -> impl Fn(&&Field) -> bool + '_ {
        move |field| !self.skip(field.ident.as_ref().unwrap())
    }
}

impl AttrsDatabase<usize> {
    pub(super) fn new_from_unnamed(fields: &FieldsUnnamed) -> syn::Result<Self> {
        let map = fields
            .unnamed
            .iter()
            .enumerate()
            .map(|(index, field)| {
                let attrs = InnerAttrs::parse(&field.attrs)?;
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

    pub(super) fn meta(&self, key: &T) -> TokenStream {
        self.map
            .get(key)
            .map(|attrs| tokenize_meta(&attrs.meta))
            .unwrap_or_else(|| {
                quote! {
                    Default::default()
                }
            })
    }

    pub(super) fn docs(&self, key: &T) -> TokenStream {
        let docs = self.map.get(key).into_iter().flat_map(|attrs| &attrs.docs);
        quote! { &[#(#docs,)*] }
    }
}

#[derive(Debug)]
pub(super) struct InnerAttrs {
    pub(super) skip: bool,
    pub(super) meta: HashMap<Ident, Expr>,
    pub(super) docs: Vec<LitStr>,
}

impl InnerAttrs {
    pub(super) fn new(docs: Vec<LitStr>) -> Self {
        Self {
            skip: Default::default(),
            meta: Default::default(),
            docs,
        }
    }

    pub(super) fn parse(attrs: &[Attribute]) -> syn::Result<Self> {
        let docs = parse_docs(attrs);

        let mut reflect_attrs = attrs
            .iter()
            .filter(|attr| attr.path.is_ident("reflect"))
            .peekable();

        let Some(attr) = reflect_attrs.next() else { return Ok(Self::new(docs)) };

        if let Some(next) = reflect_attrs.peek() {
            return Err(syn::Error::new_spanned(
                next,
                "Can only have one `#[reflect(...)]` attribute",
            ));
        }

        attr.parse_args_with(|input: ParseStream<'_>| {
            let mut field_attrs = Self::new(docs);

            while !input.is_empty() {
                let lh = input.lookahead1();
                if lh.peek(kw::skip) {
                    input.parse::<kw::skip>()?;
                    field_attrs.skip = true;
                } else if lh.peek(kw::meta) {
                    input.parse::<kw::meta>()?;
                    let content;
                    syn::parenthesized!(content in input);
                    while !content.is_empty() {
                        let ident = content.parse::<Ident>()?;
                        content.parse::<Token![=]>()?;
                        let expr = content.parse::<Expr>()?;
                        if field_attrs.meta.insert(ident.clone(), expr).is_some() {
                            return Err(syn::Error::new_spanned(
                                &ident,
                                format!("`{ident}` specified more than once"),
                            ));
                        }

                        let _ = content.parse::<Token![,]>();
                    }
                } else {
                    return Err(lh.error());
                }

                let _ = input.parse::<Token![,]>();
            }

            Ok(field_attrs)
        })
    }

    pub(super) fn meta(&self) -> TokenStream {
        tokenize_meta(&self.meta)
    }

    pub(super) fn docs(&self) -> TokenStream {
        let docs = &self.docs;
        quote! { &[#(#docs,)*] }
    }
}
