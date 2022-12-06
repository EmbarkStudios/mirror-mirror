use super::attrs::InnerAttrs;
use super::attrs::ItemAttrs;
use super::Generics;
use crate::stringify;
use proc_macro2::TokenStream;
use quote::quote;
use std::borrow::Cow;
use syn::DataEnum;
use syn::Fields;
use syn::Ident;
use syn::Type;

pub(super) fn expand(
    ident: &Ident,
    enum_: DataEnum,
    attrs: ItemAttrs,
    generics: &Generics<'_>,
) -> syn::Result<TokenStream> {
    let variants = VariantData::try_from_enum(&enum_)?;

    let reflect = expand_reflect(ident, &variants, &attrs, generics)?;
    let from_reflect = expand_from_reflect(ident, &variants, &attrs, generics);
    let enum_ = expand_enum(ident, &variants, generics);

    Ok(quote! {
        #reflect
        #from_reflect
        #enum_
    })
}

fn expand_reflect(
    ident: &Ident,
    variants: &[VariantData<'_>],
    attrs: &ItemAttrs,
    generics: &Generics<'_>,
) -> syn::Result<TokenStream> {
    let fn_patch = {
        let match_arms = variants.iter().filter(filter_out_skipped).map(|variant| {
            let variant_ident = &variant.ident;
            let field_names = variant.field_names();

            match &variant.fields {
                FieldsData::Named(fields) => {
                    let set_fields = fields.iter().filter(filter_out_skipped).map(|field| {
                        let ident = field.ident;
                        let ident_string = stringify(ident);
                        quote! {
                            if let Some(new_value) = enum_.field(#ident_string) {
                                #ident.patch(new_value);
                            }
                        }
                    });

                    quote! {
                        Self::#variant_ident { #(#field_names,)* } => {
                            if variant_matches {
                                #(#set_fields)*
                            }
                        }
                    }
                }
                FieldsData::Unnamed(fields) => {
                    let set_fields = fields.iter().enumerate().filter(filter_out_skipped).map(
                        |(index, field)| {
                            let ident = &field.fake_ident;
                            quote! {
                                if let Some(new_value) = enum_.field_at(#index) {
                                    #ident.patch(new_value);
                                }
                            }
                        },
                    );

                    quote! {
                        Self::#variant_ident(#(#field_names,)*) => {
                            if variant_matches {
                                #(#set_fields)*
                            }
                        }
                    }
                }
                FieldsData::Unit => {
                    quote! {
                        Self::#variant_ident => {
                            // unit variants don't have any fields to patch
                        }
                    }
                }
            }
        });

        if attrs.clone_opt_out {
            quote! {
                fn patch(&mut self, value: &dyn Reflect) {
                    if let Some(enum_) = value.reflect_ref().as_enum() {
                        if let Some(new) = Self::from_reflect(value) {
                            *self = new;
                        } else {
                            let variant_matches = self.variant_name() == enum_.variant_name();
                            match self {
                                #(#match_arms)*
                                _ => {}
                            }
                        }
                    }
                }
            }
        } else {
            quote! {
                fn patch(&mut self, value: &dyn Reflect) {
                    if let Some(new) = value.downcast_ref::<Self>() {
                        *self = new.clone();
                    } else if let Some(enum_) = value.reflect_ref().as_enum() {
                        if let Some(new) = Self::from_reflect(value) {
                            *self = new;
                        } else {
                            let variant_matches = self.variant_name() == enum_.variant_name();
                            match self {
                                #(#match_arms)*
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    };

    let fn_to_value = {
        let match_arms = variants.iter().filter(filter_out_skipped).map(|variant| {
            let variant_ident = &variant.ident;
            let variant_ident_string = stringify(variant_ident);
            let field_names = variant.field_names();

            match &variant.fields {
                FieldsData::Named(fields) => {
                    let set_fields = fields.iter().filter(filter_out_skipped).map(|field| {
                        let ident = &field.ident;
                        let ident_string = stringify(ident);
                        quote! {
                            value.set_struct_field(#ident_string, #ident.to_value());
                        }
                    });

                    quote! {
                        Self::#variant_ident { #(#field_names,)* } => {
                            let mut value = EnumValue::new_struct_variant(#variant_ident_string);
                            #(#set_fields)*
                            value.into()
                        }
                    }
                }
                FieldsData::Unnamed(fields) => {
                    let field_names = field_names.collect::<Vec<_>>();

                    let included_fields = fields
                        .iter()
                        .filter(filter_out_skipped)
                        .map(|field| &field.fake_ident);

                    quote! {
                        Self::#variant_ident(#(#field_names,)*) => {
                            let mut value = EnumValue::new_tuple_variant(#variant_ident_string);
                            #(
                                value.push_tuple_field(#included_fields.to_value());
                            )*
                            value.into()
                        }
                    }
                }
                FieldsData::Unit => {
                    quote! {
                        Self::#variant_ident => {
                            EnumValue::new_unit_variant(#variant_ident_string).into()
                        }
                    }
                }
            }
        });

        quote! {
            fn to_value(&self) -> Value {
                match self {
                    #(#match_arms)*
                    other => {
                        panic!("`Reflection::to_value` called on `{:?}` which doesn't suport reflection", other.as_reflect())
                    }
                }
            }
        }
    };

    let fn_type_info = {
        let code_for_variants = variants.iter().filter(filter_out_skipped).map(|variant| {
            let variant_ident_string = stringify(&variant.ident);
            let meta = variant.attrs.meta();
            let docs = variant.attrs.docs();

            match &variant.fields {
                FieldsData::Named(fields) => {
                    let fields = fields.iter().filter(filter_out_skipped).map(|field| {
                        let ident = &field.ident;
                        let field_name = stringify(ident);
                        let field_ty = &field.ty;
                        let meta = field.attrs.meta();
                        let docs = field.attrs.docs();
                        quote! {
                            NamedFieldNode::new::<#field_ty>(#field_name, #meta, #docs, graph)
                        }
                    });

                    quote! {
                        VariantNode::Struct(
                            StructVariantInfoNode::new(
                                #variant_ident_string,
                                &[#(#fields),*],
                                #meta,
                                #docs,
                            )
                        )
                    }
                }
                FieldsData::Unnamed(fields) => {
                    let fields = fields.iter().filter(filter_out_skipped).map(|field| {
                        let field_ty = &field.ty;
                        let meta = field.attrs.meta();
                        let docs = field.attrs.docs();
                        quote! {
                            UnnamedFieldNode::new::<#field_ty>(#meta, #docs, graph)
                        }
                    });

                    quote! {
                        VariantNode::Tuple(
                            TupleVariantInfoNode::new(
                                #variant_ident_string,
                                &[#(#fields),*],
                                #meta,
                                #docs,
                            )
                        )
                    }
                }
                FieldsData::Unit => quote! {
                    VariantNode::Unit(UnitVariantInfoNode::new(
                        #variant_ident_string,
                        #meta,
                        #docs,
                    ))
                },
            }
        });

        let meta = attrs.meta();
        let docs = attrs.docs();

        let Generics {
            impl_generics,
            type_generics,
            where_clause,
        } = generics;

        quote! {
            fn type_info(&self) -> TypeInfoRoot {
                impl #impl_generics Typed for #ident #type_generics #where_clause {
                    fn build(graph: &mut TypeInfoGraph) -> Id {
                        let variants = &[#(#code_for_variants),*];
                        graph.get_or_build_with::<Self, _>(|graph| {
                            EnumInfoNode::new::<Self>(variants, #meta, #docs)
                        })
                    }
                }

                <Self as Typed>::type_info()
            }
        }
    };

    let fn_debug = attrs.fn_debug_tokens();
    let fn_clone_reflect = attrs.fn_clone_reflect_tokens();

    let Generics {
        impl_generics,
        type_generics,
        where_clause,
    } = generics;

    Ok(quote! {
        impl #impl_generics Reflect for #ident #type_generics #where_clause {
            fn as_any(&self) -> &dyn Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn Any {
                self
            }

            fn as_reflect(&self) -> &dyn Reflect {
                self
            }

            fn as_reflect_mut(&mut self) -> &mut dyn Reflect {
                self
            }

            #fn_type_info
            #fn_patch
            #fn_to_value
            #fn_clone_reflect
            #fn_debug

            fn reflect_ref(&self) -> ReflectRef<'_> {
                ReflectRef::Enum(self)
            }

            fn reflect_mut(&mut self) -> ReflectMut<'_> {
                ReflectMut::Enum(self)
            }
        }
    })
}

fn expand_from_reflect(
    ident: &Ident,
    variants: &[VariantData<'_>],
    attrs: &ItemAttrs,
    generics: &Generics<'_>,
) -> TokenStream {
    let match_arms = variants.iter().filter(filter_out_skipped).map(|variant| {
        let variant_ident = &variant.ident;
        let variant_ident_string = stringify(&variant.ident);

        let expr = match &variant.fields {
            FieldsData::Named(fields) => {
                let set_fields = fields.iter().map(|field| {
                    let ident = &field.ident;

                    if field.skip() {
                        quote! {
                            #ident: std::default::Default::default(),
                        }
                    } else {
                        let ident_string = stringify(ident);
                        let ty = &field.ty;
                        if attrs.clone_opt_out {
                            quote! {
                                #ident: {
                                    let value = enum_.field(#ident_string)?;
                                    FromReflect::from_reflect(value)?
                                },
                            }
                        } else {
                            quote! {
                                #ident: {
                                    let value = enum_.field(#ident_string)?;
                                    if let Some(value) = value.downcast_ref::<#ty>() {
                                        value.to_owned()
                                    } else {
                                        FromReflect::from_reflect(value)?
                                    }
                                },
                            }
                        }
                    }
                });

                quote! {
                    Some(Self::#variant_ident {
                        #(#set_fields)*
                    }),
                }
            }
            FieldsData::Unnamed(fields) => {
                let set_fields = fields.iter().enumerate().map(|(idx, field)| {
                    if field.skip() {
                        quote! {
                            std::default::Default::default(),
                        }
                    } else {
                        let ty = &field.ty;
                        if attrs.clone_opt_out {
                            quote! {
                                {
                                    let value = enum_.field_at(#idx)?;
                                    FromReflect::from_reflect(value)?
                                },
                            }
                        } else {
                            quote! {
                                {
                                    let value = enum_.field_at(#idx)?;
                                    if let Some(value) = value.downcast_ref::<#ty>() {
                                        value.to_owned()
                                    } else {
                                        FromReflect::from_reflect(value)?
                                    }
                                },
                            }
                        }
                    }
                });

                quote! {
                    Some(Self::#variant_ident(#(#set_fields)*)),
                }
            }
            FieldsData::Unit => {
                quote! {
                    Some(Self::#variant_ident),
                }
            }
        };

        quote! {
            #variant_ident_string => #expr
        }
    });

    let Generics {
        impl_generics,
        type_generics,
        where_clause,
    } = generics;

    quote! {
        impl #impl_generics FromReflect for #ident #type_generics #where_clause {
            fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
                let enum_ = reflect.reflect_ref().as_enum()?;
                match enum_.variant_name() {
                    #(#match_arms)*
                    _ => None,
                }
            }
        }
    }
}

fn expand_enum(
    ident: &Ident,
    variants: &[VariantData<'_>],
    generics: &Generics<'_>,
) -> TokenStream {
    let fn_variant_name = {
        let match_arms = variants.iter().map(|variant| {
            let ident = &variant.ident;
            let ident_string = stringify(ident);
            quote! {
                Self::#ident { .. } => #ident_string,
            }
        });

        quote! {
            fn variant_name(&self) -> &str {
                match self {
                    #(#match_arms)*
                }
            }
        }
    };

    let fn_variant_kind = {
        let match_arms = variants.iter().map(|variant| {
            let ident = &variant.ident;

            let kind = match &variant.fields {
                FieldsData::Named(_) => quote! { VariantKind::Struct },
                FieldsData::Unnamed(_) => quote! { VariantKind::Tuple },
                FieldsData::Unit => quote! { VariantKind::Unit },
            };

            quote! {
                Self::#ident { .. } => #kind,
            }
        });

        quote! {
            fn variant_kind(&self) -> VariantKind {
                match self {
                    #(#match_arms)*
                }
            }
        }
    };

    let fn_field = {
        let match_arms = variants.iter().filter(filter_out_skipped).map(|variant| {
            let variant_ident = &variant.ident;

            match &variant.fields {
                FieldsData::Named(fields) => {
                    let field_names = variant.field_names();

                    let return_if_name_matches =
                        fields.iter().filter(filter_out_skipped).map(|field| {
                            let ident = &field.ident;
                            let ident_string = stringify(ident);
                            quote! {
                                if name == #ident_string {
                                    return Some(#ident);
                                }
                            }
                        });

                    quote! {
                        Self::#variant_ident { #(#field_names,)* } => {
                            #(#return_if_name_matches)*
                        }
                    }
                }
                FieldsData::Unnamed(_) => quote! {
                    Self::#variant_ident(..) => return None,
                },
                FieldsData::Unit => quote! {
                    Self::#variant_ident => return None,
                },
            }
        });

        quote! {
            #[allow(unused_variables, unreachable_code)]
            fn field(&self, name: &str) -> Option<&dyn Reflect> {
                match self {
                    #(#match_arms)*
                    _ => {}
                }

                None
            }

        }
    };

    let fn_field_mut = {
        let match_arms = variants.iter().filter(filter_out_skipped).map(|variant| {
            let variant_ident = &variant.ident;

            match &variant.fields {
                FieldsData::Named(fields) => {
                    let field_names = variant.field_names();

                    let return_if_name_matches =
                        fields.iter().filter(filter_out_skipped).map(|field| {
                            let ident = &field.ident;
                            let ident_string = stringify(ident);
                            quote! {
                                if name == #ident_string {
                                    return Some(#ident);
                                }
                            }
                        });

                    quote! {
                        Self::#variant_ident { #(#field_names,)* } => {
                            #(#return_if_name_matches)*
                        },
                    }
                }
                FieldsData::Unnamed(_) => quote! {
                    Self::#variant_ident(..) => return None,
                },
                FieldsData::Unit => quote! {
                    Self::#variant_ident => return None,
                },
            }
        });

        quote! {
            #[allow(unused_variables, unreachable_code)]
            fn field_mut(&mut self, name: &str) -> Option<&mut dyn Reflect> {
                match self {
                    #(#match_arms)*
                    _ => {}
                }

                None
            }

        }
    };

    let fn_field_at = {
        let match_arms = variants.iter().filter(filter_out_skipped).map(|variant| {
            let variant_ident = &variant.ident;

            match &variant.fields {
                FieldsData::Named(_) => {
                    quote! {
                        Self::#variant_ident { .. } => return None,
                    }
                }
                FieldsData::Unnamed(fields) => {
                    let field_names = variant.field_names();

                    let return_if_index_matches = fields
                        .iter()
                        .enumerate()
                        .filter(filter_out_skipped)
                        .map(|(idx, field)| {
                            let field_name = &field.fake_ident;
                            quote! {
                                if #idx == index {
                                    return Some(#field_name.as_reflect());
                                }
                            }
                        });

                    quote! {
                        Self::#variant_ident(#(#field_names,)*) => {
                            #(#return_if_index_matches)*
                        },
                    }
                }
                FieldsData::Unit => quote! {
                    Self::#variant_ident => return None,
                },
            }
        });

        quote! {
            #[allow(unused_variables, unreachable_code)]
            fn field_at(&self, index: usize) -> Option<&dyn Reflect> {
                match self {
                    #(#match_arms)*
                    _ => {}
                }

                None
            }
        }
    };

    let fn_field_at_mut = {
        let match_arms = variants.iter().filter(filter_out_skipped).map(|variant| {
            let variant_ident = &variant.ident;

            match &variant.fields {
                FieldsData::Named(_) => {
                    quote! {
                        Self::#variant_ident { .. } => return None,
                    }
                }
                FieldsData::Unnamed(fields) => {
                    let field_names = variant.field_names();

                    let return_if_index_matches = fields
                        .iter()
                        .enumerate()
                        .filter(filter_out_skipped)
                        .map(|(idx, field)| {
                            let field_name = &field.fake_ident;
                            quote! {
                                if #idx == index {
                                    return Some(#field_name.as_reflect_mut());
                                }
                            }
                        });

                    quote! {
                        Self::#variant_ident(#(#field_names,)*) => {
                            #(#return_if_index_matches)*
                        },
                    }
                }
                FieldsData::Unit => quote! {
                    Self::#variant_ident => return None,
                },
            }
        });

        quote! {
            #[allow(unused_variables, unreachable_code)]
            fn field_at_mut(&mut self, index: usize) -> Option<&mut dyn Reflect> {
                match self {
                    #(#match_arms)*
                    _ => {}
                }

                None
            }
        }
    };

    let fn_fields = {
        let match_arms = variants.iter().filter(filter_out_skipped).map(|variant| {
            let variant_ident = &variant.ident;
            let field_names = variant.field_names().collect::<Vec<_>>();

            match &variant.fields {
                FieldsData::Named(fields) => {
                    let code_for_fields = fields.iter().filter(filter_out_skipped).map(|field| {
                        let ident = &field.ident;
                        let field = stringify(ident);
                        quote! {
                            (#field, #ident.as_reflect()),
                        }
                    });

                    quote! {
                        Self::#variant_ident { #(#field_names,)* } => {
                            let iter = [#(#code_for_fields)*];
                            VariantFieldIter::new_struct_variant(iter)
                        },
                    }
                }
                FieldsData::Unnamed(fields) => {
                    let included_fields = fields
                        .iter()
                        .filter(filter_out_skipped)
                        .map(|field| &field.fake_ident);
                    quote! {
                        Self::#variant_ident(#(#field_names,)*) => {
                            let iter = [#(#included_fields.as_reflect(),)*];
                            VariantFieldIter::new_tuple_variant(iter)
                        },
                    }
                }
                FieldsData::Unit => quote! {
                    Self::#variant_ident => {
                        VariantFieldIter::empty()
                    },
                },
            }
        });

        quote! {
            fn fields(&self) -> VariantFieldIter<'_> {
                match self {
                    #(#match_arms)*
                    _ => VariantFieldIter::empty(),
                }
            }
        }
    };

    let fn_fields_mut = {
        let match_arms = variants.iter().filter(filter_out_skipped).map(|variant| {
            let variant_ident = &variant.ident;
            let field_names = variant.field_names().collect::<Vec<_>>();

            match &variant.fields {
                FieldsData::Named(fields) => {
                    let code_for_fields = fields.iter().filter(filter_out_skipped).map(|field| {
                        let ident = &field.ident;
                        let field = stringify(ident);
                        quote! {
                            (#field, #ident.as_reflect_mut()),
                        }
                    });

                    quote! {
                        Self::#variant_ident { #(#field_names,)* } => {
                            let iter = [#(#code_for_fields)*];
                            VariantFieldIterMut::new_struct_variant(iter)
                        },
                    }
                }
                FieldsData::Unnamed(fields) => {
                    let included_fields = fields
                        .iter()
                        .filter(filter_out_skipped)
                        .map(|field| &field.fake_ident);

                    quote! {
                        Self::#variant_ident(#(#field_names,)*) => {
                            let iter = [#(#included_fields.as_reflect_mut(),)*];
                            VariantFieldIterMut::new_tuple_variant(iter)
                        },
                    }
                }
                FieldsData::Unit => quote! {
                    Self::#variant_ident => {
                        VariantFieldIterMut::empty()
                    },
                },
            }
        });

        quote! {
            fn fields_mut(&mut self) -> VariantFieldIterMut<'_> {
                match self {
                    #(#match_arms)*
                    _ => VariantFieldIterMut::empty(),
                }
            }
        }
    };

    let Generics {
        impl_generics,
        type_generics,
        where_clause,
    } = generics;

    quote! {
        impl #impl_generics Enum for #ident #type_generics #where_clause {
            #fn_variant_name
            #fn_variant_kind
            #fn_field
            #fn_field_mut
            #fn_field_at
            #fn_field_at_mut
            #fn_fields
            #fn_fields_mut
        }
    }
}

struct VariantData<'a> {
    ident: &'a Ident,
    attrs: InnerAttrs,
    fields: FieldsData<'a>,
}

impl<'a> VariantData<'a> {
    fn try_from_enum(enum_: &'a DataEnum) -> syn::Result<Vec<Self>> {
        enum_
            .variants
            .iter()
            .map(|variant| -> syn::Result<VariantData<'_>> {
                let fields: FieldsData<'a> = match &variant.fields {
                    Fields::Named(fields) => {
                        let fields = fields
                            .named
                            .iter()
                            .map(|field| {
                                let ident = field.ident.as_ref().unwrap();
                                let ty = &field.ty;
                                let attrs = InnerAttrs::parse(&field.attrs)?;

                                Ok(NamedField { ident, ty, attrs })
                            })
                            .collect::<syn::Result<Vec<_>>>()?;

                        FieldsData::Named(fields)
                    }
                    Fields::Unnamed(fields) => {
                        let fields = fields
                            .unnamed
                            .iter()
                            .enumerate()
                            .map(|(index, field)| {
                                let ty = &field.ty;
                                let attrs = InnerAttrs::parse(&field.attrs)?;
                                let fake_ident = quote::format_ident!("field_{index}");

                                Ok(UnnamedField {
                                    ty,
                                    attrs,
                                    fake_ident,
                                })
                            })
                            .collect::<syn::Result<Vec<_>>>()?;

                        FieldsData::Unnamed(fields)
                    }
                    Fields::Unit => FieldsData::Unit,
                };

                let attrs = InnerAttrs::parse(&variant.attrs)?;

                Ok(VariantData {
                    ident: &variant.ident,
                    fields,
                    attrs,
                })
            })
            .collect::<syn::Result<Vec<_>>>()
    }
}

impl<'a> VariantData<'a> {
    fn field_names<'this>(&'this self) -> Box<dyn Iterator<Item = Cow<'a, Ident>> + 'this> {
        match &self.fields {
            FieldsData::Named(fields) => {
                Box::new(fields.iter().map(|field| Cow::Borrowed(field.ident)))
            }
            FieldsData::Unnamed(fields) => Box::new(
                fields
                    .iter()
                    .enumerate()
                    .map(|(index, _)| Cow::Owned(quote::format_ident!("field_{index}"))),
            ),
            FieldsData::Unit => Box::new(std::iter::empty()),
        }
    }
}

enum FieldsData<'a> {
    Named(Vec<NamedField<'a>>),
    Unnamed(Vec<UnnamedField<'a>>),
    Unit,
}

struct NamedField<'a> {
    ident: &'a Ident,
    ty: &'a Type,
    attrs: InnerAttrs,
}

struct UnnamedField<'a> {
    ty: &'a Type,
    attrs: InnerAttrs,
    fake_ident: Ident,
}

fn filter_out_skipped<T>(skippable: &T) -> bool
where
    T: Skippable,
{
    !skippable.skip()
}

trait Skippable {
    fn skip(&self) -> bool;
}

impl<T> Skippable for &T
where
    T: Skippable + ?Sized,
{
    fn skip(&self) -> bool {
        T::skip(self)
    }
}

impl<T> Skippable for (usize, T)
where
    T: Skippable + ?Sized,
{
    fn skip(&self) -> bool {
        self.1.skip()
    }
}

impl Skippable for VariantData<'_> {
    fn skip(&self) -> bool {
        self.attrs.skip
    }
}

impl Skippable for NamedField<'_> {
    fn skip(&self) -> bool {
        self.attrs.skip
    }
}

impl Skippable for UnnamedField<'_> {
    fn skip(&self) -> bool {
        self.attrs.skip
    }
}
